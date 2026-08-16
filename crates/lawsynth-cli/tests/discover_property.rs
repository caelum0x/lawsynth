use std::fs;

use lawsynth_bundle::read_world;
use lawsynth_cli::run;

// A tiny standards-conformant fixture writer for the supported native Parquet
// subset: uncompressed DATA_PAGE/PLAIN/DOUBLE columns. Keeping this in the
// This integration test exercises the actual binary ingestion path end to end.
fn varint(mut value: u64, out: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 127) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 128;
        }
        out.push(byte);
        if value == 0 {
            return;
        }
    }
}

fn thrift_i32(value: i32, out: &mut Vec<u8>) {
    varint(((value << 1) ^ (value >> 31)) as u32 as u64, out);
}

fn thrift_i64(value: i64, out: &mut Vec<u8>) {
    varint(((value << 1) ^ (value >> 63)) as u64, out);
}

fn plain_double_page(values: &[f64]) -> Vec<u8> {
    let body = values.iter().flat_map(|value| value.to_le_bytes()).collect::<Vec<_>>();
    let length = i32::try_from(body.len()).unwrap();
    let mut page = vec![0x15]; // PageHeader.type = DATA_PAGE.
    thrift_i32(0, &mut page);
    page.push(0x15); // PageHeader.uncompressed_page_size.
    thrift_i32(length, &mut page);
    page.push(0x15); // PageHeader.compressed_page_size.
    thrift_i32(length, &mut page);
    page.push(0x2c); // PageHeader.data_page_header.
    page.push(0x15); // DataPageHeader.num_values.
    thrift_i32(i32::try_from(values.len()).unwrap(), &mut page);
    page.push(0x15); // DataPageHeader.encoding = PLAIN.
    thrift_i32(0, &mut page);
    page.extend([0, 0]); // DataPageHeader terminator, PageHeader terminator.
    page.extend(body);
    page
}

fn parquet_column_chunk(name: &str, offset: usize, size: usize, values: usize) -> Vec<u8> {
    let mut chunk = vec![0x3c]; // ColumnChunk.meta_data.
    chunk.push(0x15); // ColumnMetaData.type = DOUBLE.
    thrift_i32(5, &mut chunk);
    chunk.push(0x29); // path_in_schema list.
    chunk.push(0x18); // one BINARY item.
    varint(name.len() as u64, &mut chunk);
    chunk.extend(name.as_bytes());
    chunk.push(0x15); // codec = UNCOMPRESSED.
    thrift_i32(0, &mut chunk);
    chunk.push(0x16); // num_values.
    thrift_i64(values as i64, &mut chunk);
    chunk.push(0x26); // total_compressed_size.
    thrift_i64(size as i64, &mut chunk);
    chunk.push(0x26); // data_page_offset.
    thrift_i64(offset as i64, &mut chunk);
    chunk.extend([0, 0]);
    chunk
}

fn native_plain_double_parquet(time: &[f64], values: &[f64]) -> Vec<u8> {
    let time_page = plain_double_page(time);
    let values_page = plain_double_page(values);
    let time_offset = 4;
    let value_offset = time_offset + time_page.len();
    let mut metadata = vec![0x49, 0x1c]; // FileMetaData.row_groups: one struct.
    metadata.push(0x19); // RowGroup.columns.
    metadata.push(0x2c); // two STRUCT elements.
    metadata.extend(parquet_column_chunk("t", time_offset, time_page.len(), time.len()));
    metadata.extend(parquet_column_chunk("x", value_offset, values_page.len(), values.len()));
    metadata.extend([0, 0]);
    let mut file = b"PAR1".to_vec();
    file.extend(time_page);
    file.extend(values_page);
    file.extend(&metadata);
    file.extend(u32::try_from(metadata.len()).unwrap().to_le_bytes());
    file.extend(b"PAR1");
    file
}

#[test]
fn discover_command_writes_a_simulatable_world() {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-discover-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    let csv = directory.join("growth.csv");
    let contents = (0..101)
        .map(|step| {
            let time = step as f64 * 0.01;
            format!("{time},{:.17e}", (2.0 * time).exp())
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&csv, format!("t,x\n{contents}\n")).unwrap();
    let bundle = directory.join("growth.lsworld");

    let output = run(&[
        "discover".to_owned(),
        csv.display().to_string(),
        "--time".to_owned(),
        "t".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--output".to_owned(),
        bundle.display().to_string(),
        "--trigonometric".to_owned(),
        "--savgol-window".to_owned(),
        "5".to_owned(),
        "--smooth-radius".to_owned(),
        "1".to_owned(),
        "--bootstrap".to_owned(),
        "3".to_owned(),
    ])
    .unwrap();

    assert!(output.starts_with("discovered world:"));
    assert!(read_world(&bundle).is_ok());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn discover_command_accepts_tv_regularized_differentiation() {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-tvreg-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    let csv = directory.join("growth.csv");
    let contents = (0..101)
        .map(|step| {
            let time = step as f64 * 0.01;
            format!("{time},{:.17e}", (2.0 * time).exp())
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&csv, format!("t,x\n{contents}\n")).unwrap();
    let bundle = directory.join("growth.lsworld");

    let output = run(&[
        "discover".to_owned(),
        csv.display().to_string(),
        "--time".to_owned(),
        "t".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--output".to_owned(),
        bundle.display().to_string(),
        "--tvreg-lambda".to_owned(),
        "0.001".to_owned(),
        "--tvreg-iterations".to_owned(),
        "150".to_owned(),
    ])
    .unwrap();

    assert!(output.starts_with("discovered world:"));
    assert!(read_world(&bundle).is_ok());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn tsv_ingestion_flows_through_discovery_bundle_inspection_and_simulation() {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-tsv-e2e-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    let observations = directory.join("growth.tsv");
    let records = (0..101)
        .map(|step| {
            let time = step as f64 * 0.01;
            format!("{time}\t{:.17e}", (1.5 * time).exp())
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&observations, format!("t\tx\n{records}\n")).unwrap();
    let bundle = directory.join("growth.lsworld");

    let discovered = run(&[
        "discover".to_owned(),
        observations.display().to_string(),
        "--time".to_owned(),
        "t".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--output".to_owned(),
        bundle.display().to_string(),
        "--degree".to_owned(),
        "1".to_owned(),
    ])
    .unwrap();
    assert!(discovered.starts_with("discovered world:"));
    assert!(
        run(&["inspect".to_owned(), bundle.display().to_string()])
            .unwrap()
            .starts_with("continuous world: 1 states")
    );

    let simulated = run(&[
        "simulate".to_owned(),
        bundle.display().to_string(),
        "--initial".to_owned(),
        "x=1".to_owned(),
        "--start".to_owned(),
        "0".to_owned(),
        "--end".to_owned(),
        "0.2".to_owned(),
        "--step".to_owned(),
        "0.05".to_owned(),
    ])
    .unwrap();
    assert_eq!(simulated.lines().count(), 6);
    assert!(simulated.lines().last().unwrap().contains(','));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn parquet_ingestion_flows_through_discovery_bundle_inspection_and_simulation() {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-parquet-e2e-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    let observations = directory.join("growth.parquet");
    let time = (0..101).map(|step| step as f64 * 0.01).collect::<Vec<_>>();
    let values = time.iter().map(|time| (1.25 * time).exp()).collect::<Vec<_>>();
    fs::write(&observations, native_plain_double_parquet(&time, &values)).unwrap();
    let bundle = directory.join("growth.lsworld");

    assert!(
        run(&[
            "discover".to_owned(),
            observations.display().to_string(),
            "--time".to_owned(),
            "t".to_owned(),
            "--state".to_owned(),
            "x".to_owned(),
            "--output".to_owned(),
            bundle.display().to_string(),
            "--degree".to_owned(),
            "1".to_owned(),
        ])
        .unwrap()
        .starts_with("discovered world:")
    );
    assert!(
        run(&["inspect".to_owned(), bundle.display().to_string()])
            .unwrap()
            .starts_with("continuous world: 1 states")
    );
    let simulated = run(&[
        "simulate".to_owned(),
        bundle.display().to_string(),
        "--initial".to_owned(),
        "x=1".to_owned(),
        "--start".to_owned(),
        "0".to_owned(),
        "--end".to_owned(),
        "0.2".to_owned(),
        "--step".to_owned(),
        "0.05".to_owned(),
    ])
    .unwrap();
    assert_eq!(simulated.lines().count(), 6);
    fs::remove_dir_all(directory).unwrap();
}
