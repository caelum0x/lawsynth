use lawsynth_plugin_api::ResourceLimits;
use lawsynth_plugin_host::ResourceMeter;

fn main() {
    let mut meter = ResourceMeter::new(ResourceLimits {
        max_requests: 2,
        max_output_bytes: 32,
        ..Default::default()
    })
    .unwrap();
    meter.begin_request().unwrap();
    meter.record_output(12).unwrap();
    println!("first request accepted after {:?}", meter.elapsed());
}
