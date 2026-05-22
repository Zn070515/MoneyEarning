use wasm_core::{DataFrame, IndError, IndicatorOutput, IndicatorMeta, ParamDef};
use std::collections::HashMap;

pub mod trend;
pub mod momentum;
pub mod volatility;
pub mod volume;
pub mod cycle;
pub mod composite;
pub mod custom;
pub mod candles;
pub mod pro;
pub mod benches;

pub fn compute(name: &str, df: &DataFrame, params: &HashMap<String, f64>) -> Result<Vec<IndicatorOutput>, IndError> {
    match name {
        // Overlap / Trend
        "sma" => trend::sma::compute(df, params),
        "ema" => trend::ema::compute(df, params),
        "wma" => trend::wma::compute(df, params),
        "dema" => trend::dema::compute(df, params),
        "tema" => trend::tema::compute(df, params),
        "trima" => trend::trima::compute(df, params),
        "rma" => trend::rma::compute(df, params),
        "linreg" => trend::linreg::compute(df, params),
        "hl2" => trend::hl2::compute(df, params),
        "hlc3" => trend::hlc3::compute(df, params),
        "ohlc4" => trend::ohlc4::compute(df, params),
        "midpoint" => trend::midpoint::compute(df, params),
        "midprice" => trend::midprice::compute(df, params),
        "wcp" => trend::wcp::compute(df, params),
        "hilo" => trend::hilo::compute(df, params),
        "macd" => trend::macd::compute(df, params),
        "adx" => trend::adx::compute(df, params),
        "psar" => trend::psar::compute(df, params),
        "aroon" => trend::aroon::compute(df, params),
        "supertrend" => trend::supertrend::compute(df, params),
        "dpo" => trend::dpo::compute(df, params),
        "vortex" => trend::vortex::compute(df, params),
        "qstick" => trend::qstick::compute(df, params),
        "decreasing" => trend::decreasing::compute(df, params),
        "increasing" => trend::increasing::compute(df, params),
        "chop" => trend::chop::compute(df, params),

        // Momentum
        "rsi" => momentum::rsi::compute(df, params),
        "kdj" => momentum::kdj::compute(df, params),
        "stoch" => momentum::stoch::compute(df, params),
        "cci" => momentum::cci::compute(df, params),
        "williams_r" => momentum::williams_r::compute(df, params),
        "roc" => momentum::roc::compute(df, params),
        "mom" => momentum::mom::compute(df, params),
        "mfi" => momentum::mfi::compute(df, params),
        "fisher" => momentum::fisher::compute(df, params),
        "apo" => momentum::apo::compute(df, params),
        "ppo" => momentum::ppo::compute(df, params),
        "bias" => momentum::bias::compute(df, params),
        "psl" => momentum::psl::compute(df, params),
        "bop" => momentum::bop::compute(df, params),
        "slope" => momentum::slope::compute(df, params),
        "inertia" => momentum::inertia::compute(df, params),
        "er" => momentum::er::compute(df, params),
        "brar" => momentum::brar::compute(df, params),

        // Volatility
        "bb" => volatility::bb::compute(df, params),
        "atr" => volatility::atr::compute(df, params),
        "kc" => volatility::kc::compute(df, params),
        "dc" => volatility::dc::compute(df, params),
        "natr" => volatility::natr::compute(df, params),
        "true_range" => volatility::true_range::compute(df, params),
        "massi" => volatility::massi::compute(df, params),
        "rvi" => volatility::rvi::compute(df, params),
        "thermo" => volatility::thermo::compute(df, params),
        "ui" => volatility::ui::compute(df, params),

        // Volume
        "obv" => volume::obv::compute(df, params),
        "vwap" => volume::vwap::compute(df, params),
        "vwma" => volume::vwma::compute(df, params),
        "ad" => volume::ad::compute(df, params),
        "cmf" => volume::cmf::compute(df, params),
        "adosc" => volume::adosc::compute(df, params),
        "eom" => volume::eom::compute(df, params),
        "pvt" => volume::pvt::compute(df, params),
        "pvol" => volume::pvol::compute(df, params),

        // Cycles
        "ebsw" => cycle::ebsw::compute(df, params),
        "seasonal" => cycle::seasonal::compute(df, params),

        // Statistics
        "stdev" => composite::stdev::compute(df, params),
        "variance" => composite::variance::compute(df, params),
        "zscore" => composite::zscore::compute(df, params),
        "mad" => composite::mad::compute(df, params),
        "median" => composite::median::compute(df, params),

        // Performance
        "percent_return" => composite::percent_return::compute(df, params),
        "trend_return" => composite::trend_return::compute(df, params),

        // Custom tools
        "fib_retracement" => custom::fib_retracement::compute(df, params),
        "sr_detect" => custom::sr_detect::compute(df, params),
        "correlation" => custom::correlation::compute(df, params),
        "range_stats" => custom::range_stats::compute(df, params),

        // Candles
        "cdl_doji" => candles::cdl_doji(df, params),
        "cdl_hammer" => candles::cdl_hammer(df, params),
        "cdl_inverted_hammer" => candles::cdl_inverted_hammer(df, params),
        "cdl_hanging_man" => candles::cdl_hanging_man(df, params),
        "cdl_shooting_star" => candles::cdl_shooting_star(df, params),
        "cdl_engulfing" => candles::cdl_engulfing(df, params),
        "cdl_harami" => candles::cdl_harami(df, params),
        "cdl_piercing" => candles::cdl_piercing(df, params),
        "cdl_dark_cloud_cover" => candles::cdl_dark_cloud_cover(df, params),
        "cdl_morning_star" => candles::cdl_morning_star(df, params),
        "cdl_evening_star" => candles::cdl_evening_star(df, params),
        "cdl_three_white_soldiers" => candles::cdl_three_white_soldiers(df, params),
        "cdl_three_black_crows" => candles::cdl_three_black_crows(df, params),
        "cdl_marubozu" => candles::cdl_marubozu(df, params),
        "cdl_inside" => candles::cdl_inside(df, params),

        // PRO indicators — try them before returning InvalidName
        _ => pro::compute(name, df, params),
    }
}

pub fn compute_many(
    specs: &[(String, HashMap<String, f64>)],
    df: &DataFrame,
) -> Result<Vec<Vec<IndicatorOutput>>, IndError> {
    specs.iter().map(|(name, params)| compute(name, df, params)).collect()
}

fn p_def(name: &str, default: f64, min: f64, max: f64) -> ParamDef {
    ParamDef { name: name.into(), default, min, max, step: 1.0 }
}

macro_rules! meta {
    ($name:literal, $cn:literal, $cat:literal, $params:expr, $free:literal, $tdx:expr, $desc:literal) => {
        IndicatorMeta {
            name: $name.into(), name_cn: $cn.into(), category: $cat.into(),
            params: $params, is_free: $free, tdx_equivalent: $tdx,
            description: $desc.into(),
        }
    };
}

pub fn list_all() -> Vec<IndicatorMeta> {
    vec![
        // === Overlap 均线/叠加类 (Free 18) ===
        meta!("sma", "简单移动均线", "均线/叠加类", vec![p_def("period",20.0,2.0,500.0)], true, Some("MA".into()),
            "N周期简单移动平均线"),
        meta!("ema", "指数移动均线", "均线/叠加类", vec![p_def("period",20.0,2.0,500.0)], true, Some("EXPMA".into()),
            "N周期指数加权移动平均线"),
        meta!("wma", "加权移动均线", "均线/叠加类", vec![p_def("period",20.0,2.0,500.0)], true, Some("WMA".into()),
            "线性加权移动平均线，近期价格权重大"),
        meta!("dema", "双指数移动均线", "均线/叠加类", vec![p_def("period",20.0,2.0,500.0)], true, None,
            "Mulloy 1994，2×EMA - EMA(EMA)，降低滞后"),
        meta!("tema", "三指数移动均线", "均线/叠加类", vec![p_def("period",20.0,2.0,500.0)], true, None,
            "Mulloy 1994，3×EMA - 3×EMA(EMA) + EMA(EMA(EMA))"),
        meta!("trima", "三角移动均线", "均线/叠加类", vec![p_def("period",20.0,2.0,500.0)], true, None,
            "权重成三角分布的平滑移动平均线"),
        meta!("vwap", "成交量加权均价", "均线/叠加类", vec![], true, Some("VWAP".into()),
            "日内机构交易基准价"),
        meta!("vwma", "成交量加权均线", "均线/叠加类", vec![p_def("period",20.0,2.0,200.0)], true, Some("AMV".into()),
            "以成交量为权重的移动平均线"),
        meta!("hl2", "高低价均值", "均线/叠加类", vec![], true, None,
            "(High+Low)/2，最简单的价格代表"),
        meta!("hlc3", "典型价格", "均线/叠加类", vec![], true, Some("TYP".into()),
            "(High+Low+Close)/3"),
        meta!("ohlc4", "OHLC均值", "均线/叠加类", vec![], true, None,
            "(Open+High+Low+Close)/4"),
        meta!("midpoint", "中点", "均线/叠加类", vec![p_def("period",10.0,2.0,200.0)], true, None,
            "N周期最高最低价的均值"),
        meta!("midprice", "中间价", "均线/叠加类", vec![p_def("period",10.0,2.0,200.0)], true, None,
            "N周期最高最低价的中点"),
        meta!("wcp", "加权收盘价", "均线/叠加类", vec![], true, None,
            "(High+Low+2×Close)/4"),
        meta!("rma", "Wilder平滑均线", "均线/叠加类", vec![p_def("period",14.0,2.0,100.0)], true, None,
            "Wilder 1978平滑方式，RSI/ATR的基础"),
        meta!("linreg", "线性回归线", "均线/叠加类", vec![p_def("period",20.0,2.0,200.0)], true, Some("LINEAR".into()),
            "N周期线性回归趋势线终点值"),
        meta!("hilo", "Gann高/低激活线", "均线/叠加类", vec![p_def("period",20.0,2.0,200.0)], true, None,
            "以SMA为基准的高低激活线"),
        meta!("supertrend", "超级趋势", "均线/叠加类",
            vec![p_def("period",10.0,2.0,100.0), p_def("multiplier",3.0,1.0,10.0)],
            true, None, "Oliver 2008，基于ATR的趋势跟踪叠加线"),

        // === Trend 趋势/方向类 (Free 10) ===
        meta!("macd", "平滑异同均线", "趋势/方向类",
            vec![p_def("fast",12.0,2.0,200.0), p_def("slow",26.0,2.0,200.0), p_def("signal",9.0,2.0,50.0)],
            true, Some("MACD".into()), "DIF/DEA/柱三线，最经典的趋势指标"),
        meta!("adx", "平均趋向指数", "趋势/方向类", vec![p_def("period",14.0,2.0,100.0)], true, Some("ADX".into()),
            "Wilder的ADX/PDI/MDI三线，判断趋势强弱"),
        meta!("psar", "抛物线SAR", "趋势/方向类",
            vec![p_def("af_step",0.02,0.01,0.1), p_def("af_max",0.2,0.1,1.0)],
            true, Some("SAR".into()), "停损转向点，趋势反转信号"),
        meta!("aroon", "阿隆指标", "趋势/方向类", vec![p_def("period",14.0,2.0,100.0)], true, None,
            "Tushar Chande 1995，判断趋势是否存在"),
        meta!("dpo", "去趋势价格振荡器", "趋势/方向类", vec![p_def("period",20.0,2.0,200.0)], true, Some("DPO".into()),
            "消除长期趋势后分析周期波动"),
        meta!("vortex", "漩涡指标", "趋势/方向类", vec![p_def("period",14.0,2.0,100.0)], true, None,
            "+VI/-VI交叉判定趋势反转，比ADX响应快"),
        meta!("qstick", "Q棒", "趋势/方向类", vec![p_def("period",14.0,2.0,100.0)], true, None,
            "开盘-收盘价差的均线，衡量做市压力"),
        meta!("decreasing", "持续下降线", "趋势/方向类", vec![p_def("period",3.0,2.0,20.0)], true, None,
            "价格N日连续下降标记"),
        meta!("increasing", "持续上升线", "趋势/方向类", vec![p_def("period",3.0,2.0,20.0)], true, None,
            "价格N日连续上升标记"),
        meta!("chop", "震荡指数", "趋势/方向类", vec![p_def("period",14.0,2.0,100.0)], true, None,
            "0-100，越高越震荡（choppy），越低越趋势"),

        // === Momentum ===
        meta!("rsi", "相对强弱指数", "动量/振荡类", vec![p_def("period",14.0,2.0,100.0)], true, Some("RSI".into()),
            "Wilder 1978，>70超买/<30超卖"),
        meta!("kdj", "KDJ指标", "动量/振荡类",
            vec![p_def("n",9.0,2.0,100.0), p_def("m1",3.0,2.0,50.0), p_def("m2",3.0,2.0,50.0)],
            true, Some("KDJ".into()), "%K/%D/%J三线，A股最常用"),
        meta!("stoch", "随机指标", "动量/振荡类",
            vec![p_def("k_period",14.0,2.0,100.0), p_def("k_slow",3.0,1.0,50.0), p_def("d_period",3.0,1.0,50.0)],
            true, Some("KD".into()), "Stochastic %K/%D，期货经典指标"),
        meta!("cci", "商品通道指数", "动量/振荡类", vec![p_def("period",20.0,2.0,100.0)], true, Some("CCI".into()),
            "Lambert 1980，衡量价格偏离统计均值"),
        meta!("williams_r", "威廉指标", "动量/振荡类", vec![p_def("period",14.0,2.0,100.0)], true, Some("WR".into()),
            "Williams %R，-80以下超卖，-20以上超买"),
        meta!("roc", "变动率", "动量/振荡类", vec![p_def("period",12.0,1.0,200.0)], true, Some("ROC".into()),
            "N周期涨跌幅百分比"),
        meta!("mom", "动量线", "动量/振荡类", vec![p_def("period",12.0,1.0,200.0)], true, Some("MOM".into()),
            "N周期价格差"),
        meta!("mfi", "资金流量指数", "动量/振荡类", vec![p_def("period",14.0,2.0,100.0)], true, Some("MFI".into()),
            "结合价格+成交量的RSI变体"),
        meta!("fisher", "费雪变换", "动量/振荡类", vec![p_def("period",10.0,2.0,100.0)], true, None,
            "Fisher Transform，规范化价格到高斯分布"),
        meta!("apo", "绝对价格振荡器", "动量/振荡类",
            vec![p_def("fast",12.0,2.0,200.0), p_def("slow",26.0,2.0,200.0)],
            true, None, "快EMA−慢EMA，用差值衡量动量"),
        meta!("ppo", "百分比价格振荡器", "动量/振荡类",
            vec![p_def("fast",12.0,2.0,200.0), p_def("slow",26.0,2.0,200.0)],
            true, None, "(快EMA/慢EMA−1)×100%，相对动量"),
        meta!("bias", "乖离率", "动量/振荡类", vec![p_def("period",6.0,2.0,200.0)], true, Some("BIAS".into()),
            "(收盘价/均线−1)×100%，衡量偏离程度"),
        meta!("psl", "心理线", "动量/振荡类", vec![p_def("period",12.0,2.0,200.0)], true, Some("PSY".into()),
            "N日内上涨天数比例，衡量市场情绪"),
        meta!("bop", "力量平衡", "动量/振荡类", vec![], true, None,
            "(收−开)/(高−低)，多空力量对比"),
        meta!("slope", "斜率", "动量/振荡类", vec![p_def("period",20.0,2.0,200.0)], true, Some("ACCER".into()),
            "时间序列线性回归斜率"),
        meta!("inertia", "惯性", "动量/振荡类", vec![p_def("period",14.0,2.0,100.0)], true, None,
            "基于RVGI平滑的动量持续性度量"),
        meta!("er", "效率比", "动量/振荡类", vec![p_def("period",10.0,2.0,100.0)], true, None,
            "净位移/总路程，衡量趋势效率"),
        meta!("brar", "BRAR人气意愿", "动量/振荡类", vec![p_def("period",26.0,2.0,200.0)], true, Some("BRAR".into()),
            "A股传统指标，BR(人气)+AR(意愿)双线"),

        // === Volatility ===
        meta!("bb", "布林带", "波动/通道类",
            vec![p_def("period",20.0,2.0,200.0), p_def("stddev",2.0,1.0,5.0)],
            true, Some("BOLL".into()), "中轨(SMA)+上下轨(±Nσ)，Bollinger 1983"),
        meta!("atr", "平均真实波幅", "波动/通道类", vec![p_def("period",14.0,2.0,100.0)], true, Some("ATR".into()),
            "Wilder的Average True Range，衡量波动性"),
        meta!("kc", "肯特纳通道", "波动/通道类",
            vec![p_def("period",20.0,2.0,200.0), p_def("atr_period",10.0,2.0,100.0), p_def("multiplier",2.0,0.5,5.0)],
            true, None, "EMA+ATR×N倍构成通道"),
        meta!("dc", "唐奇安通道", "波动/通道类", vec![p_def("period",20.0,2.0,200.0)], true, None,
            "N日最高/最低价通道，海龟交易法则核心"),
        meta!("natr", "归一化ATR", "波动/通道类", vec![p_def("period",14.0,2.0,100.0)], true, None,
            "ATR/收盘价×100%，跨品种可比"),
        meta!("true_range", "真实波幅", "波动/通道类", vec![], true, Some("TR".into()),
            "max(H−L, |H−C_prev|, |L−C_prev|)"),
        meta!("massi", "质量指数", "波动/通道类", vec![p_def("period",25.0,2.0,200.0)], true, Some("MASS".into()),
            "高低价差均线比值累积，>27预示反转"),
        meta!("rvi", "相对波动率指数", "波动/通道类", vec![p_def("period",14.0,2.0,100.0)], true, None,
            "基于标准差比值的波动率度量"),
        meta!("thermo", "Elder温度计", "波动/通道类", vec![p_def("period",20.0,2.0,200.0)], true, None,
            "Elder 2002，波动率冷热度"),
        meta!("ui", "溃疡指数", "波动/通道类", vec![p_def("period",14.0,2.0,100.0)], true, None,
            "Martin 1987，回撤平方的均方根"),

        // === Volume ===
        meta!("obv", "能量潮", "成交量类", vec![], true, Some("OBV".into()),
            "On-Balance Volume，Joseph Granville 1963"),
        meta!("ad", "累计/派发线", "成交量类", vec![], true, Some("AD".into()),
            "Accumulation/Distribution，Chaikin A/D Line"),
        meta!("cmf", "蔡金资金流", "成交量类", vec![p_def("period",20.0,2.0,200.0)], true, None,
            "Chaikin Money Flow，A/D的N周期平滑版本"),
        meta!("adosc", "Chaikin振荡器", "成交量类",
            vec![p_def("fast",3.0,1.0,20.0), p_def("slow",10.0,2.0,50.0)],
            true, Some("CHO".into()), "快/慢A/D线的MACD"),
        meta!("eom", "易动性", "成交量类", vec![p_def("period",14.0,2.0,100.0)], true, Some("EMV".into()),
            "Arms 1978，成交量的价格推动效率"),
        meta!("pvt", "量价趋势", "成交量类", vec![], true, Some("VPT".into()),
            "成交量×(今日收−昨日收)/昨日收的累积"),
        meta!("pvol", "价格成交量", "成交量类", vec![], true, None,
            "成交量×收盘价的简单乘积"),

        // === Statistics 统计/分布类 (Free 5) ===
        meta!("stdev", "标准差", "统计/分布类", vec![p_def("period",20.0,2.0,200.0)], true, None,
            "价格波动率的一阶度量"),
        meta!("variance", "方差", "统计/分布类", vec![p_def("period",20.0,2.0,200.0)], true, None,
            "标准差的平方"),
        meta!("zscore", "Z分数", "统计/分布类", vec![p_def("period",20.0,2.0,200.0)], true, None,
            "当前价格距均值几个标准差的偏离"),
        meta!("mad", "平均绝对偏差", "统计/分布类", vec![p_def("period",20.0,2.0,200.0)], true, None,
            "比标准差对异常值更稳健"),
        meta!("median", "中位数", "统计/分布类", vec![p_def("period",20.0,2.0,200.0)], true, None,
            "鲁棒中心度量，不受极端值影响"),

        // === Cycles 周期/傅里叶类 (Free 2) ===
        meta!("ebsw", "正弦波指标", "周期/傅里叶类", vec![p_def("period",40.0,10.0,200.0)], true, None,
            "Ehlers基于带通滤波的周期检测"),
        meta!("seasonal", "月度效应", "周期/傅里叶类", vec![p_def("mode",0.0,0.0,1.0)], true, None,
            "周/月平均收益日历（0=周,1=月）"),

        // === Performance 绩效/回撤类 (Free 2) ===
        meta!("percent_return", "百分比收益", "绩效/回撤类", vec![p_def("period",1.0,1.0,252.0)], true, None,
            "N周期百分比收益"),
        meta!("trend_return", "趋势收益", "绩效/回撤类", vec![p_def("period",20.0,2.0,200.0)], true, None,
            "当前价格相对N周期均线的偏离百分比"),

        // === Custom Tools 特色工具 (Free 4) ===
        meta!("fib_retracement", "Fibonacci回调/扩展", "特色工具类",
            vec![p_def("period",100.0,20.0,500.0)], true, None,
            "0/0.236/0.382/0.5/0.618/0.786/1.0 + 1.272/1.618"),
        meta!("sr_detect", "支撑/阻力检测", "特色工具类",
            vec![p_def("period",20.0,5.0,100.0), p_def("threshold",3.0,1.5,10.0)],
            true, None, "基于转折点+成交量密集区的S/R自动识别"),
        meta!("correlation", "简单相关性", "特色工具类",
            vec![p_def("period",20.0,5.0,200.0), p_def("col",0.0,0.0,99.0)],
            true, None, "两只股票价格走势的N周期Pearson r"),
        meta!("range_stats", "区间统计", "特色工具类",
            vec![p_def("start",0.0,0.0,99999.0), p_def("end",100.0,1.0,99999.0)],
            true, None, "指定区间涨跌幅+最大回撤+均值ATR"),

        // === Candles K线形态识别 (Free 15) ===
        meta!("cdl_doji", "十字星", "K线形态识别", vec![], true, None, "实体极小，多空力量均衡"),
        meta!("cdl_hammer", "锤子线", "K线形态识别", vec![], true, None, "下跌后长下影线，潜在反转看涨"),
        meta!("cdl_inverted_hammer", "倒锤子", "K线形态识别", vec![], true, None, "下跌后长上影线，潜在反转看涨"),
        meta!("cdl_hanging_man", "吊颈线", "K线形态识别", vec![], true, None, "上涨后长下影线，潜在反转看跌"),
        meta!("cdl_shooting_star", "射击之星", "K线形态识别", vec![], true, None, "上涨后长上影线，潜在反转看跌"),
        meta!("cdl_engulfing", "吞没形态", "K线形态识别", vec![], true, None, "实体完全吞没前一日实体，强反转信号"),
        meta!("cdl_harami", "孕线", "K线形态识别", vec![], true, None, "实体完全被前一日实体包含，反转预警"),
        meta!("cdl_piercing", "穿刺线", "K线形态识别", vec![], true, None, "下跌后收阳且过前阴实体50%，看涨"),
        meta!("cdl_dark_cloud_cover", "乌云盖顶", "K线形态识别", vec![], true, None, "上涨后收阴且破前阳实体50%，看跌"),
        meta!("cdl_morning_star", "晨星", "K线形态识别", vec![], true, None, "阴+小星+阳三K线组合，底部反转"),
        meta!("cdl_evening_star", "黄昏之星", "K线形态识别", vec![], true, None, "阳+小星+阴三K线组合，顶部反转"),
        meta!("cdl_three_white_soldiers", "三白兵", "K线形态识别", vec![], true, None, "连续三阳且重心上移，强看涨"),
        meta!("cdl_three_black_crows", "三乌鸦", "K线形态识别", vec![], true, None, "连续三阴且重心下移，强看跌"),
        meta!("cdl_marubozu", "光头光脚", "K线形态识别", vec![], true, None, "几乎没有影线的饱满实体"),
        meta!("cdl_inside", "内含线", "K线形态识别", vec![], true, None, "整根K线被前一K线高低价包含"),

        // ═══════════════════════════════════════════════
        // PRO 专业版指标 (is_free = false, 需授权)
        // ═══════════════════════════════════════════════

        // === PRO 均线/叠加类 ===
        meta!("kama", "Kaufman自适应均线", "均线/叠加类",
            vec![p_def("period",10.0,2.0,200.0), p_def("fast",2.0,2.0,50.0), p_def("slow",30.0,2.0,200.0)],
            false, None, "Kaufman AMA，基于效率比ER的自适应平滑"),
        meta!("hma", "Hull移动均线", "均线/叠加类",
            vec![p_def("period",20.0,2.0,200.0)],
            false, None, "Alan Hull的HMA，极低滞后性移动均线"),
        meta!("t3", "T3移动均线", "均线/叠加类",
            vec![p_def("period",10.0,2.0,100.0), p_def("v",0.7,0.1,1.0)],
            false, None, "Ehler的T3，6重EMA平滑+体量因数"),
        meta!("vidya", "动态自适应均线", "均线/叠加类",
            vec![p_def("period",9.0,2.0,100.0)],
            false, Some("VIDYA".into()), "Chande的VIDYA，用CMO绝对值做平滑常数"),
        meta!("alma", "ALMA均线", "均线/叠加类",
            vec![p_def("period",9.0,2.0,200.0), p_def("offset",0.85,0.0,1.0), p_def("sigma",6.0,1.0,20.0)],
            false, None, "Arnaud Legoux MA，高斯分布加权+偏移"),
        meta!("lsma", "最小二乘均线", "均线/叠加类",
            vec![p_def("period",25.0,2.0,200.0)],
            false, None, "OLS线性回归终点线，专业趋势分析"),
        meta!("frama", "分形自适应均线", "均线/叠加类",
            vec![p_def("period",16.0,4.0,200.0)],
            false, None, "Ehler的FRAMA，分形维数自适应"),
        meta!("chandelier_exit", "吊灯止损", "均线/叠加类",
            vec![p_def("period",22.0,5.0,100.0), p_def("atr_period",22.0,2.0,100.0), p_def("multiplier",3.0,1.0,5.0)],
            false, None, "LeBeau吊灯止损，做多/做空双轨"),
        meta!("jma", "Jurik移动均线", "均线/叠加类",
            vec![p_def("period",14.0,2.0,200.0), p_def("phase",0.0,-100.0,100.0)],
            false, None, "Jurik自适应低滞后平滑线"),
        meta!("gmma", "顾比多均线", "均线/叠加类",
            vec![],
            false, None, "Guppy MMA 6+6=12条，短/长期组交叉分析"),

        // === PRO 趋势/方向类 ===
        meta!("ichimoku", "一目均衡表", "趋势/方向类",
            vec![p_def("tenkan",9.0,2.0,100.0), p_def("kijun",26.0,2.0,200.0), p_def("senkou_b",52.0,2.0,300.0)],
            false, Some("ICHIMOKU".into()), "日式一目均衡5线+云层系统"),
        meta!("alligator", "鳄鱼线", "趋势/方向类",
            vec![p_def("jaw",13.0,2.0,100.0), p_def("teeth",8.0,2.0,100.0), p_def("lips",5.0,2.0,100.0)],
            false, None, "Williams鳄鱼线，jaw/teeth/lips三线"),
        meta!("fractals", "分形标记", "趋势/方向类",
            vec![p_def("period",2.0,1.0,5.0)],
            false, None, "Williams五柱分形，上/下分形突破点"),
        meta!("rainbow_ma", "彩虹均线", "趋势/方向类",
            vec![],
            false, None, "10条EMA从2到20同时显示"),
        meta!("ao", "动量震荡器", "趋势/方向类",
            vec![],
            false, Some("AO".into()), "Williams AO，5-34柱状图动量"),
        meta!("ac", "加速度震荡器", "趋势/方向类",
            vec![],
            false, Some("AC".into()), "Williams AC，AO的加速度指标"),

        // === PRO 动量/振荡类 ===
        meta!("tsi", "真实强度指数", "动量/振荡类",
            vec![p_def("long",25.0,2.0,100.0), p_def("short",13.0,2.0,50.0), p_def("signal",7.0,2.0,30.0)],
            false, None, "Blau的TSI，双平滑动量归一化"),
        meta!("smi", "随机动量指数", "动量/振荡类",
            vec![p_def("k_period",5.0,2.0,50.0), p_def("k_smooth",3.0,1.0,20.0), p_def("d_period",3.0,1.0,20.0), p_def("signal",5.0,1.0,30.0)],
            false, None, "Blau的SMI，闭合区间内随机振荡"),
        meta!("stoch_rsi", "随机RSI", "动量/振荡类",
            vec![p_def("period",14.0,2.0,100.0), p_def("k",3.0,1.0,20.0), p_def("d",3.0,1.0,20.0)],
            false, None, "RSI的Stochastic转换，更敏感"),
        meta!("trix", "三重指数振荡器", "动量/振荡类",
            vec![p_def("period",15.0,2.0,100.0), p_def("signal",9.0,2.0,50.0)],
            false, Some("TRIX".into()), "TRIX+信号线，三重平滑去噪"),
        meta!("elder_ray", "Elder射线", "动量/振荡类",
            vec![p_def("period",13.0,2.0,200.0)],
            false, None, "Elder多空力，牛力+熊力双线"),
        meta!("ultimate_osc", "终极振荡器", "动量/振荡类",
            vec![p_def("short",7.0,2.0,50.0), p_def("mid",14.0,2.0,100.0), p_def("long",28.0,2.0,200.0)],
            false, Some("UOS".into()), "Williams三时框加权振荡器"),
        meta!("rmi", "相对动量指数", "动量/振荡类",
            vec![p_def("period",14.0,2.0,100.0), p_def("momentum_period",5.0,1.0,20.0)],
            false, None, "RSI的动量增强版，Momentum替代差值"),
        meta!("ergotic", "Ergotic振荡器", "动量/振荡类",
            vec![p_def("period",20.0,2.0,100.0), p_def("signal",5.0,1.0,50.0)],
            false, None, "Chande的CSI，循环/趋势自适应"),

        // === PRO 波动/通道类 ===
        meta!("hist_vol", "历史波动率", "波动/通道类",
            vec![p_def("period",21.0,5.0,252.0), p_def("annualize",252.0,1.0,365.0)],
            false, None, "收盘价对数收益率年化波动率"),
        meta!("gk_vol", "Garman-Klass波动率", "波动/通道类",
            vec![p_def("period",21.0,5.0,252.0), p_def("annualize",252.0,1.0,365.0)],
            false, None, "OHLC全信息波动率估计"),
        meta!("parkinson_vol", "Parkinson波动率", "波动/通道类",
            vec![p_def("period",21.0,5.0,252.0), p_def("annualize",252.0,1.0,365.0)],
            false, None, "仅用最高最低价的波动率估计"),
        meta!("rs_vol", "Rogers-Satchell波动率", "波动/通道类",
            vec![p_def("period",21.0,5.0,252.0), p_def("annualize",252.0,1.0,365.0)],
            false, None, "消除漂移偏差的OHLC波动率"),
        meta!("yz_vol", "Yang-Zhang波动率", "波动/通道类",
            vec![p_def("period",21.0,5.0,252.0), p_def("annualize",252.0,1.0,365.0)],
            false, None, "最全面OHLC波动率，跳空兼容"),
        meta!("hurst", "Hurst指数", "波动/通道类",
            vec![p_def("period",100.0,20.0,500.0)],
            false, None, "R/S分析法Hurst指数，>0.5趋势/<0.5均值回归"),
        meta!("threshold_bands", "波动阈值带", "波动/通道类",
            vec![p_def("period",20.0,5.0,100.0), p_def("multiplier",2.0,1.0,5.0)],
            false, None, "基于波动率形态的动态支撑阻力"),
        meta!("regression_channel", "回归通道", "波动/通道类",
            vec![p_def("period",20.0,5.0,200.0), p_def("deviation",2.0,1.0,4.0)],
            false, None, "OLS回归带，±N标准误通道"),

        // === PRO 成交量类 ===
        meta!("kvo", "Klinger成交量振荡器", "成交量类",
            vec![p_def("fast",34.0,10.0,100.0), p_def("slow",55.0,20.0,200.0), p_def("signal",13.0,2.0,50.0)],
            false, None, "Klinger VF+信号线，量价背离"),
        meta!("vfi", "量流指数", "成交量类",
            vec![p_def("period",130.0,20.0,300.0), p_def("coef",0.2,0.05,0.5), p_def("vcoef",2.5,1.0,5.0)],
            false, None, "Katsanos的VFI，成交量方向流"),
        meta!("force_index", "强力指数", "成交量类",
            vec![p_def("period",13.0,1.0,100.0)],
            false, Some("FI".into()), "Elder的FI，价差×成交量平滑"),
        meta!("mfi2", "市场促进指数", "成交量类",
            vec![],
            false, None, "Williams MFI，量价效率(非资金流指数)"),
        meta!("volume_osc", "成交量振荡器", "成交量类",
            vec![p_def("fast",5.0,2.0,50.0), p_def("slow",10.0,5.0,100.0)],
            false, None, "量MACD，快慢量均线差值百分比"),
        meta!("emv_v2", "增强易动性", "成交量类",
            vec![p_def("period",14.0,2.0,100.0)],
            false, Some("EMV_P".into()), "增强Ease of Movement，量价效率"),
        meta!("volume_regime", "量能形态", "成交量类",
            vec![p_def("period",20.0,10.0,100.0), p_def("threshold",1.5,1.0,3.0)],
            false, None, "高/低量能形态检测，Z-score分类"),

        // === PRO 周期/傅里叶类 ===
        meta!("hilbert_sine", "Hilbert正弦波", "周期/傅里叶类",
            vec![p_def("period",7.0,2.0,50.0)],
            false, None, "Ehler主导周期检测+正弦/超前正弦"),
        meta!("mesa_sine", "MESA正弦波", "周期/傅里叶类",
            vec![p_def("period",14.0,5.0,100.0)],
            false, None, "MESA自适应正弦/余弦波"),
        meta!("cg", "重心指标", "周期/傅里叶类",
            vec![p_def("period",10.0,2.0,50.0)],
            false, None, "Ehler的重心指标，近乎零滞后"),
        meta!("phasor", "相位计", "周期/傅里叶类",
            vec![p_def("period",14.0,5.0,100.0)],
            false, None, "Ehler相位计，判断趋势/循环状态"),

        // === PRO 特色工具类 ===
        meta!("pivots", "地板交易枢轴", "特色工具类",
            vec![p_def("period",1.0,1.0,10.0)],
            false, None, "经典P-S1/S2/S3-R1/R2/R3七线系统"),
        meta!("decision_point", "决策点评分", "特色工具类",
            vec![p_def("rsi_period",14.0,2.0,100.0), p_def("macd_fast",12.0,2.0,200.0), p_def("macd_slow",26.0,2.0,200.0), p_def("bb_period",20.0,2.0,200.0)],
            false, None, "RSI+MACD+BB三合一信号评分-100~+100"),
    ]
}

pub fn list_by_category(cat: &str) -> Vec<IndicatorMeta> {
    list_all().into_iter().filter(|m| m.category == cat).collect()
}

pub fn metadata(name: &str) -> Option<IndicatorMeta> {
    list_all().into_iter().find(|m| m.name == name)
}
