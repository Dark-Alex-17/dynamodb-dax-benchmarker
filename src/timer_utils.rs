#[macro_export]
macro_rules! time {
    ($x:expr) => {{
        let start = std::time::Instant::now();
        let _result = $x;
        serde_json::Number::from(start.elapsed().as_millis())
    }};

    ($resp:ident, $x:expr) => {{
        let start = std::time::Instant::now();
        let $resp = $x;
        (serde_json::Number::from(start.elapsed().as_millis()), $resp)
    }};
}
