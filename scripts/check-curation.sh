#!/usr/bin/env bash
# Fails unless every built package matches its curation record: file name,
# version in the published index and SHA-256 of the WASM payload, and every
# package has a record. Run after scripts/build.sh. See "Adding a source" in
# README.md.
set -euo pipefail
cd "$(dirname "$0")/.."

records=curation/entries.json
status=0

for aix in public/sources/*.aix; do
	file="$(basename "$aix")"
	actual="$(unzip -p "$aix" Payload/main.wasm | shasum -a 256 | awk '{print $1}')"
	recorded="$(jq -r --arg file "$file" '.entries[] | select(.package.file == $file) | .package.payloadSha256' "$records")"
	if [[ -z "$recorded" ]]; then
		echo "::error::$file has no curation record in $records"
		status=1
	elif [[ "$recorded" != "$actual" ]]; then
		echo "::error::$file payload SHA-256 is $actual but the curation record pins $recorded"
		status=1
	else
		echo "ok  $file  payload $actual"
	fi
done

# Every record must still correspond to a built package, at the pinned version.
while IFS=$'\t' read -r id version file; do
	if [[ ! -f "public/sources/$file" ]]; then
		echo "::error::record for $id has no built package $file"
		status=1
		continue
	fi
	listed="$(jq -r --arg id "$id" '.sources[] | select(.id == $id) | .version' public/index.min.json)"
	if [[ "$listed" != "$version" ]]; then
		echo "::error::$id is version $listed in index.min.json but its record pins version $version"
		status=1
	fi
done < <(jq -r '.entries[] | [.id, (.version | tostring), .package.file] | @tsv' "$records")

exit "$status"
