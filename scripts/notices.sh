#!/usr/bin/env bash
# Writes to stdout the licences of what's built into the exiftool-rs program,
# which go wherever it does: its own, ExifTool's, the C libraries it compiles
# in (PCRE2 with its JIT, zlib) and the Rust crates it links, for every
# target, so that one file serves each platform's program. The release job
# publishes it beside them; the crates' sources must be fetched first
# (`cargo fetch --locked`).

set -euo pipefail
cd "$(dirname "$0")/.."

registry="${CARGO_HOME:-$HOME/.cargo}/registry/src"
# A crate's source, as cargo fetched it.
source_of() {
	ls -d "$registry"/*/"$1-$2" 2>/dev/null | head -1
}
rule() {
	printf '%s\n\n' "------------------------------------------------------------------------"
}
section() {
	rule
	printf '%s\n\n' "$1"
	shift
	for file in "$@"; do tr -d '\r' <"$file" && printf '\n'; done
}
# The comment at the top of a C file, where PCRE2 and its JIT keep their
# licence, having no licence file in the crate.
licence_comment() {
	tr -d '\r' <"$1" | sed -n '1,/\*\//p'
	printf '\n'
}

version=$(cargo pkgid -p exiftool-rs)
version=${version##*[@#]}
pcre2=$(cargo tree --locked -e normal --target all --prefix none -f '{p}' -i pcre2-sys | head -1 | sed 's/ (.*//')
pcre2_dir=$(source_of pcre2-sys "${pcre2##* v}")
libz=$(cargo tree --locked -e normal --target all --prefix none -f '{p}' -i libz-sys | head -1 | sed 's/ (.*//')
libz_dir=$(source_of libz-sys "${libz##* v}")
pcre2_h="$pcre2_dir/upstream/include/pcre2.h"
pcre2_version="$(sed -n 's/^#define PCRE2_MAJOR *\([0-9]*\).*/\1/p' "$pcre2_h").$(sed -n 's/^#define PCRE2_MINOR *\([0-9]*\).*/\1/p' "$pcre2_h")"

section "exiftool-rs $version (https://github.com/julund/exiftool-rs): GPL 3.0 or later" LICENSE

rule
cat <<'EOF'
ExifTool (https://exiftool.org/), by Phil Harvey, whose modules are embedded
in the program and run by it

This is free software; you can redistribute it and/or modify it under the
same terms as Perl itself: the GNU General Public License, version 1 or
later, or the Artistic License.

EOF

rule
printf '%s\n\n' "PCRE2 $pcre2_version (https://github.com/PCRE2Project/pcre2), built into the program by $pcre2: BSD 3-clause"
licence_comment "$pcre2_dir/upstream/src/pcre2_compile.c"
printf '%s\n\n' "Its JIT compiler, SLJIT: BSD 2-clause"
licence_comment "$pcre2_dir/upstream/deps/sljit/sljit_src/sljitLir.h"

section "zlib, built into the program by $libz: zlib License" "$libz_dir/src/zlib/LICENSE"

# The Rust crates built into it, each with the licence it says and the
# licence files its source has. Proc macros and build scripts run as it's
# built, and aren't in it.
cargo tree --locked -e normal,no-proc-macro --target all --prefix none -f '{p}|{l}' |
	sed 's/ (.*)//' | sort -u |
	while IFS='|' read -r crate licence; do
		name=${crate% v*}
		crate_version=${crate##* v}
		[ "$name" = exiftool-rs ] && continue
		dir=$(source_of "$name" "$crate_version")
		if [ -z "$dir" ]; then
			echo "notices.sh: $name $crate_version isn't fetched; run cargo fetch --locked" >&2
			exit 1
		fi
		files=()
		for file in "$dir"/LICENSE* "$dir"/LICENCE* "$dir"/license* "$dir"/COPYING* "$dir"/UNLICENSE*; do
			[ -f "$file" ] && files+=("$file")
		done
		section "The Rust crate $name $crate_version: ${licence:-as its source says}" "${files[@]}"
	done
