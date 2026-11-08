#!/usr/bin/env bash
# Builds every source into public/ (the tree GitHub Pages serves):
#   public/index.min.json, public/index.json, public/sources/*.aix, public/icons/*.png
# Needs the pinned Rust toolchain and the Aidoku CLI on PATH.
set -euo pipefail
cd "$(dirname "$0")/.."

rm -rf public
for source in sources/*; do
	[[ -f "$source/Cargo.toml" ]] || continue
	echo "::group::Packaging $source"
	(cd "$source" && aidoku package)
	echo "::endgroup::"
done

aidoku verify sources/*/package.aix
aidoku build sources/*/package.aix --name "nemu example sources"
# Copy each entry's contentLicense and contentLicenseNote (curation/entries.json)
# into the index. They are extra fields: Aidoku parsers that do not know them
# ignore them.
for index in public/index.json public/index.min.json; do
	jq -c --slurpfile curation curation/entries.json '
		($curation[0].entries | map({key: .id, value: ({contentLicense, contentLicenseNote} | with_entries(select(.value)))}) | from_entries) as $licenses
		| .sources |= map(. + ($licenses[.id] // {}))
	' "$index" > "$index.tmp"
	mv "$index.tmp" "$index"
done
cp site/index.html public/index.html
touch public/.nojekyll
