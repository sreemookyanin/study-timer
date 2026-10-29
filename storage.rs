use mysql::prelude::*;
use mysql::Pool;

pub fn save_session(url: &str, category: &str, seconds: u32) -> Result<(), String> {
    let pool = Pool::new(url).map_err(|e| e.to_string())?;
    let mut conn = pool.get_conn().map_err(|e| e.to_string())?;
    conn.exec_drop(
        "INSERT INTO sessions (category, seconds) VALUES (?, ?)",
        (category, seconds),
    )
    .map_err(|e| e.to_string())
}


#[derive(Debug, Clone)]
pub struct SessionRow {
    pub category: String,
    pub seconds: u32,
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub recent: Vec<SessionRow>,
    pub totals: Vec<(String, u32)>,
}

pub fn load_stats(url: &str) -> Result<Stats, String> {
    let pool = Pool::new(url).map_err(|e| e.to_string())?;
    let mut conn = pool.get_conn().map_err(|e| e.to_string())?;

    let recent = conn
        .query_map(
            "SELECT category, seconds FROM sessions ORDER BY id DESC LIMIT 10",
            |(category, seconds): (String, u32)| SessionRow { category, seconds },
        )
        .map_err(|e| e.to_string())?;

    let totals: Vec<(String, u32)> = conn
        .query(
            "SELECT category, CAST(SUM(seconds) AS UNSIGNED) \
             FROM sessions GROUP BY category ORDER BY category",
        )
        .map_err(|e| e.to_string())?;

    Ok(Stats { recent, totals })
}
