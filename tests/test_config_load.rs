// tests/test_config_load.rs
// 测试配置加载功能

#[test]
fn test_load_rated_load_table() {
    use std::fs;

    // 读取 CSV 文件
    let content = fs::read_to_string("config/rated_load_table.csv").expect("无法读取 CSV 文件");

    println!("CSV 文件内容:");
    println!("{}", content);

    // 使用 csv crate 解析
    let mut reader = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .from_reader(content.as_bytes());

    // 打印表头
    if let Ok(headers) = reader.headers() {
        println!("\n表头: {:?}", headers);
    }

    // 解析数据行
    #[derive(Debug, serde::Deserialize)]
    struct RatedLoadEntry {
        #[serde(rename = "radius_m")]
        radius: f64,
        #[serde(rename = "rated_load_ton")]
        rated_load: f64,
    }

    let mut count = 0;
    for result in reader.deserialize() {
        match result {
            Ok(entry) => {
                let entry: RatedLoadEntry = entry;
                println!(
                    "条目 {}: radius={}, rated_load={}",
                    count, entry.radius, entry.rated_load
                );
                count += 1;
            }
            Err(e) => {
                panic!("解析失败: {}", e);
            }
        }
    }

    assert!(count > 0, "应该至少有一条数据");
    println!("\n成功解析 {} 条数据", count);
}
