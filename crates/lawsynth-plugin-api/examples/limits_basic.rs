use lawsynth_plugin_api::ResourceLimits;

fn main() {
    let limits = ResourceLimits {
        max_cpu_millis: 1_000,
        max_memory_bytes: 64 * 1024 * 1024,
        max_output_bytes: 1024 * 1024,
        max_requests: 10,
    };
    limits.validate().expect("a bounded plugin configuration");
    println!("plugin accepts at most {} requests", limits.max_requests);
}
