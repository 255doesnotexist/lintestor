// File: main.rs
// Description: 主模块，负责调用调度器、汇总模块和Markdown报告生成模块。

mod scheduler;
mod aggregator;
mod markdown_report;
mod utils;

fn main() {
    // let distros = ["debian", "gentoo", "opensuse", "arch", "freebsd", "openbsd"];
    let distros = ["debian"];
    // let packages = ["mariadb", "postgresql", "sqlite", "apache", "haproxy", "lighttpd", "nginx", "squid", "varnish", "python", "libmemcached", "redis", "numpy", "scipy", "zookeeper", "openssl", "docker", "runc", "clang", "cmake", "gcc", "gdb", "llvm", "nodejs", "ocaml", "erlang", "golang", "openjdk", "perl", "python", "ruby", "rust"];
    // let packages = ["gcc", "cmake", "gdb", "llvm", "nodejs"];
    let packages = ["gcc", "cmake"];

    for distro in &distros {
        for package in &packages {
            match scheduler::run_test("localhost", 2222, "root", Some("root"), distro, package) {
                Ok(_) => println!("Test passed for {}/{}", distro, package),
                Err(e) => println!("Test failed for {}/{}: {}", distro, package, e),
            }
        }
    }

    if let Err(e) = aggregator::aggregate_reports(&distros, &packages) {
        eprintln!("Failed to aggregate reports: {}", e);
    }

    if let Err(e) = markdown_report::generate_markdown_report(&distros, &packages) {
        eprintln!("Failed to generate markdown report: {}", e);
    }
}
