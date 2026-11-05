use exiftool_rs::interp::{Interp, Scope, Want};

#[test]
fn sysread_reads_once_and_writes_at_offset() {
    let path_buf = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("sysread-{}.bin", std::process::id()));
    std::fs::write(&path_buf, b"123456").unwrap();
    let path = path_buf
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('\'', "\\'");
    let code = format!(
        "open my $fh, '<', '{path}' or die $!; \
         my $buf = 'abcdef'; \
         my $n = sysread($fh, $buf, 2, 2); \
         my $readbuf = ''; \
         my $r = read($fh, $readbuf, 2); \
         my $last = sysread($fh, $buf, 2, 4); \
         my $eof = sysread($fh, $buf, 2, 1); \
         \"$n:$buf:$r:$readbuf:$last:$eof\""
    );
    let mut it = Interp::new();
    let result = it
        .eval_string(code.as_bytes(), "main", &Scope::new(None), Want::Scalar)
        .unwrap();
    let _ = std::fs::remove_file(path_buf);
    assert_eq!(result.last().unwrap().to_text(), "2:ab1256:2:34:2:0");
}
