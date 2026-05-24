use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::MutexGuard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: i64,
    pub name: String,
    pub stock_id: i64,
    pub stock_code: Option<String>,
    pub stock_name: Option<String>,
    pub condition_type: String, // price_breakout | ma_cross | volume_spike
    pub params: String,         // JSON
    pub enabled: bool,
    pub last_triggered: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTrigger {
    pub rule: AlertRule,
    pub message: String,
    pub current_value: f64,
    pub threshold_value: f64,
}

// ── CRUD ──

pub fn create_alert(
    guard: &MutexGuard<'_, Option<Connection>>,
    name: &str, stock_id: i64, condition_type: &str, params: &str,
) -> Result<i64, String> {
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB未初始化".to_string())
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO alert_rules (name, stock_id, condition_type, params) VALUES (?1,?2,?3,?4)",
        params![name, stock_id, condition_type, params],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn list_alerts(
    guard: &MutexGuard<'_, Option<Connection>>,
) -> Result<Vec<AlertRule>, String> {
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB未初始化".to_string())
        .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.name, a.stock_id, s.code, s.name, a.condition_type,
                    a.params, a.enabled, a.last_triggered, a.created_at
             FROM alert_rules a
             LEFT JOIN stocks s ON a.stock_id = s.id
             ORDER BY a.id DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(AlertRule {
                id: row.get(0)?,
                name: row.get(1)?,
                stock_id: row.get(2)?,
                stock_code: row.get(3)?,
                stock_name: row.get(4)?,
                condition_type: row.get(5)?,
                params: row.get(6)?,
                enabled: row.get::<_, i32>(7)? != 0,
                last_triggered: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn update_alert(
    guard: &MutexGuard<'_, Option<Connection>>,
    id: i64, name: &str, condition_type: &str, params: &str, enabled: bool,
) -> Result<(), String> {
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB未初始化".to_string())
        .map_err(|e| e.to_string())?;
    let affected = conn
        .execute(
            "UPDATE alert_rules SET name=?2, condition_type=?3, params=?4, enabled=?5 WHERE id=?1",
            params![id, name, condition_type, params, enabled as i32],
        )
        .map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err("预警规则不存在".into());
    }
    Ok(())
}

pub fn delete_alert(
    guard: &MutexGuard<'_, Option<Connection>>,
    id: i64,
) -> Result<(), String> {
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB未初始化".to_string())
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM alert_rules WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Condition evaluation ──

fn eval_price_breakout(prices: &[crate::DailyPrice], params: &serde_json::Value) -> Option<AlertTrigger> {
    let direction = params["direction"].as_str().unwrap_or("above");
    let threshold = params["price"].as_f64()?;
    if prices.is_empty() { return None; }
    let last = &prices[prices.len() - 1];
    let current = last.close;
    let triggered = match direction {
        "above" => current > threshold,
        "below" => current < threshold,
        _ => false,
    };
    if triggered {
        Some(AlertTrigger {
            rule: AlertRule::dummy(),
            message: format!("价格突破: {} {} ${:.2}", direction, if direction == "above" { ">" } else { "<" }, threshold),
            current_value: current,
            threshold_value: threshold,
        })
    } else {
        None
    }
}

fn eval_ma_cross(prices: &[crate::DailyPrice], params: &serde_json::Value) -> Option<AlertTrigger> {
    let short: usize = params["short_period"].as_u64().unwrap_or(5) as usize;
    let long: usize = params["long_period"].as_u64().unwrap_or(20) as usize;
    let direction = params["direction"].as_str().unwrap_or("golden");
    if short == 0 || long == 0 || short >= long { return None; }
    if prices.len() < long + 1 { return None; }
    let closes: Vec<f64> = prices.iter().map(|p| p.close).collect();
    let ma_short_now = closes[closes.len() - short..].iter().sum::<f64>() / short as f64;
    let ma_long_now = closes[closes.len() - long..].iter().sum::<f64>() / long as f64;
    let ma_short_prev = closes[closes.len() - short - 1..closes.len() - 1]
        .iter()
        .sum::<f64>() / short as f64;
    let ma_long_prev = closes[closes.len() - long - 1..closes.len() - 1]
        .iter()
        .sum::<f64>() / long as f64;
    let crossed = match direction {
        "golden" => ma_short_prev <= ma_long_prev && ma_short_now > ma_long_now,
        "dead" => ma_short_prev >= ma_long_prev && ma_short_now < ma_long_now,
        _ => false,
    };
    if crossed {
        let label = if direction == "golden" { "金叉" } else { "死叉" };
        Some(AlertTrigger {
            rule: AlertRule::dummy(),
            message: format!("均线{}: MA{} {} MA{}", label, short, if direction == "golden" { "上穿" } else { "下破" }, long),
            current_value: ma_short_now,
            threshold_value: ma_long_now,
        })
    } else {
        None
    }
}

fn eval_volume_spike(prices: &[crate::DailyPrice], params: &serde_json::Value) -> Option<AlertTrigger> {
    let multiplier = params["multiplier"].as_f64().unwrap_or(2.0);
    let lookback: usize = params["lookback"].as_u64().unwrap_or(20) as usize;
    if lookback == 0 || prices.len() < lookback + 1 { return None; }
    let last_vol = prices[prices.len() - 1].volume;
    let avg_vol = prices[prices.len() - 1 - lookback..prices.len() - 1]
        .iter()
        .map(|p| p.volume)
        .sum::<f64>() / lookback as f64;
    if avg_vol > 0.0 && last_vol > avg_vol * multiplier {
        Some(AlertTrigger {
            rule: AlertRule::dummy(),
            message: format!("成交量异常放大: {:.1}x 平均", last_vol / avg_vol),
            current_value: last_vol,
            threshold_value: avg_vol * multiplier,
        })
    } else {
        None
    }
}

// ── Check all enabled alerts ──

pub fn check_alerts(
    guard: &MutexGuard<'_, Option<Connection>>,
    today: &str,
) -> Result<Vec<AlertTrigger>, String> {
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB未初始化".to_string())
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.name, a.stock_id, s.code, s.name, a.condition_type,
                    a.params, a.enabled, a.last_triggered, a.created_at
             FROM alert_rules a
             LEFT JOIN stocks s ON a.stock_id = s.id
             WHERE a.enabled = 1",
        )
        .map_err(|e| e.to_string())?;

    let rules: Vec<AlertRule> = stmt
        .query_map([], |row| {
            Ok(AlertRule {
                id: row.get(0)?,
                name: row.get(1)?,
                stock_id: row.get(2)?,
                stock_code: row.get(3)?,
                stock_name: row.get(4)?,
                condition_type: row.get(5)?,
                params: row.get(6)?,
                enabled: row.get::<_, i32>(7)? != 0,
                last_triggered: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut triggers: Vec<AlertTrigger> = Vec::new();

    for rule in &rules {
        if rule.last_triggered.as_deref() == Some(today) {
            continue;
        }

        let params: serde_json::Value =
            serde_json::from_str(&rule.params).unwrap_or(serde_json::Value::Null);

        let mut price_stmt = conn
            .prepare(
                "SELECT id, stock_id, trade_date, open, high, low, close, volume, amount, turnover
                 FROM daily_prices WHERE stock_id = ?1 AND trade_date <= ?2
                 ORDER BY trade_date DESC LIMIT 100",
            )
            .map_err(|e| e.to_string())?;

        let prices: Vec<crate::DailyPrice> = price_stmt
            .query_map(params![rule.stock_id, today], |row| {
                Ok(crate::DailyPrice {
                    id: row.get(0)?,
                    stock_id: row.get(1)?,
                    trade_date: row.get(2)?,
                    open: row.get(3)?,
                    high: row.get(4)?,
                    low: row.get(5)?,
                    close: row.get(6)?,
                    volume: row.get(7)?,
                    amount: row.get(8)?,
                    turnover: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        // Reverse to chronological order for MA calculation
        let mut chronological = prices;
        chronological.reverse();

        if chronological.is_empty() {
            continue;
        }

        let maybe_trigger = match rule.condition_type.as_str() {
            "price_breakout" => eval_price_breakout(&chronological, &params),
            "ma_cross" => eval_ma_cross(&chronological, &params),
            "volume_spike" => eval_volume_spike(&chronological, &params),
            _ => None,
        };

        if let Some(mut trigger) = maybe_trigger {
            trigger.rule = rule.clone();
            triggers.push(trigger);
        }
    }

    // Update last_triggered for triggered rules
    for t in &triggers {
        conn.execute(
            "UPDATE alert_rules SET last_triggered=?1 WHERE id=?2",
            params![today, t.rule.id],
        )
        .ok();
    }

    Ok(triggers)
}

impl AlertRule {
    fn dummy() -> Self {
        Self {
            id: 0,
            name: String::new(),
            stock_id: 0,
            stock_code: None,
            stock_name: None,
            condition_type: String::new(),
            params: String::new(),
            enabled: false,
            last_triggered: None,
            created_at: String::new(),
        }
    }
}
