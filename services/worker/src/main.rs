use lawsynth_worker::TransportSurface;

fn main() {
    eprintln!(
        "lawsynth-worker exposes {}; queue and network serving are not linked",
        TransportSurface::LocalDirect.reason()
    );
}
