use std::collections::HashMap;
use crate::types::{Column, OHLCV};

#[derive(Debug, Clone)]
pub struct DataFrame {
    pub columns: HashMap<String, Column>,
    pub index: Vec<i64>,
    len: usize,
}

impl DataFrame {
    pub fn new(records: &[OHLCV]) -> DataFrame {
        let n = records.len();
        let mut df = DataFrame {
            columns: HashMap::new(),
            index: Vec::with_capacity(n),
            len: n,
        };

        let mut open = Vec::with_capacity(n);
        let mut high = Vec::with_capacity(n);
        let mut low = Vec::with_capacity(n);
        let mut close = Vec::with_capacity(n);
        let mut volume = Vec::with_capacity(n);
        let mut amount = Vec::with_capacity(n);
        let mut turnover = Vec::with_capacity(n);
        let mut dates = Vec::with_capacity(n);

        for r in records.iter() {
            open.push(r.open);
            high.push(r.high);
            low.push(r.low);
            close.push(r.close);
            volume.push(r.volume);
            amount.push(r.amount.unwrap_or(0.0));
            turnover.push(r.turnover.unwrap_or(0.0));
            dates.push(r.trade_date.clone());
        }

        df.columns.insert("open".to_string(), Column::F64(open));
        df.columns.insert("high".to_string(), Column::F64(high));
        df.columns.insert("low".to_string(), Column::F64(low));
        df.columns.insert("close".to_string(), Column::F64(close));
        df.columns.insert("volume".to_string(), Column::F64(volume));
        df.columns.insert("amount".to_string(), Column::F64(amount));
        df.columns.insert("turnover".to_string(), Column::F64(turnover));
        df.columns.insert("date".to_string(), Column::String(dates));
        df.index = (0..n as i64).collect();

        df
    }

    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.get(name)
    }

    pub fn columns_map(&self) -> &HashMap<String, Column> {
        &self.columns
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn slice(&self, start: usize, end: usize) -> DataFrame {
        let end = end.min(self.len);
        let mut new_columns = HashMap::new();
        for (k, v) in &self.columns {
            let sliced = v.f64_slice(start, end);
            new_columns.insert(k.clone(), sliced);
        }
        let idx: Vec<i64> = (start as i64..end as i64).collect();
        DataFrame {
            columns: new_columns,
            index: idx,
            len: end - start,
        }
    }

    pub fn add_column(&mut self, name: &str, data: Column) {
        self.len = self.len.max(data.len());
        self.columns.insert(name.to_string(), data);
    }

    pub fn get_f64(&self, col: &str, row: usize) -> Option<f64> {
        self.column(col).and_then(|c| c.f64_values()).and_then(|v| v.get(row).copied())
    }
}

impl std::ops::Index<usize> for DataFrame {
    type Output = i64;
    fn index(&self, idx: usize) -> &i64 {
        &self.index[idx]
    }
}
