#!/usr/bin/env python3
"""Checks the public-domain sources against the institutions' own APIs.

  scripts/check-public-domain.py            check every listed file and volume
  scripts/check-public-domain.py --update   also regenerate Little Nemo's strips.tsv

Commons: a file passes when the API reports License=pd, Copyrighted=False and its
wikitext has a PD-* licence template. NDL: a volume passes when its IIIF manifest
reports Access Restrictions=PDM. Prints one line per volume or year and exits 1 on
any failure. It needs the network, so it runs by hand before a release, not in CI.
Requests are sent one at a time, as both institutions ask.
"""
import argparse
import collections
import http.client
import json
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STRIPS = ROOT / "sources/en.littlenemo/src/strips.tsv"
WORKS = ROOT / "sources/ja.ndlpublicdomain/src/works.json"
USER_AGENT = "nemu-source-registry/1 (https://github.com/nemu-pm/source-registry)"
COMMONS_API = "https://commons.wikimedia.org/w/api.php"
NEMO_YEARS = range(1905, 1912)
# Full strips; crops and details of a strip have longer names and do not match.
NEMO_FILE = re.compile(
    r"Little Nemo(?: in Slumberland)? \(?(\d{4}-\d{2}-\d{2})\)?\.(?:jpe?g|png)", re.I
)
BATCH = 50


def get_json(url, tries=4):
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(tries):
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return json.load(response)
        except urllib.error.HTTPError as error:
            # A 404 is an answer: NDL's IIIF API has no manifest for a restricted item.
            if error.code not in (429, 500, 502, 503, 504) or attempt == tries - 1:
                raise
        except (OSError, http.client.HTTPException, ValueError):
            if attempt == tries - 1:
                raise
        time.sleep(15 * (attempt + 1))


def commons(**params):
    params.update(format="json", formatversion="2", maxlag="5")
    result = get_json(f"{COMMONS_API}?{urllib.parse.urlencode(params)}")
    time.sleep(0.5)
    return result


def nemo_listing():
    """Every full strip in the yearly categories, as {date: file name}."""
    strips = {}
    for year in NEMO_YEARS:
        cont = {}
        while True:
            result = commons(
                action="query",
                list="categorymembers",
                cmtitle=f"Category:Little Nemo in Slumberland, {year}",
                cmtype="file",
                cmlimit="500",
                **cont,
            )
            for member in result["query"]["categorymembers"]:
                name = member["title"].removeprefix("File:")
                match = NEMO_FILE.fullmatch(name)
                if match:
                    strips.setdefault(match[1], name)
            if "continue" not in result:
                break
            cont = result["continue"]
    return strips


def nemo_licence_failures(files):
    failures = {}
    for start in range(0, len(files), BATCH):
        titles = [f"File:{name}" for name in files[start : start + BATCH]]
        result = commons(
            action="query",
            prop="imageinfo|revisions",
            iiprop="extmetadata",
            iiextmetadatafilter="License|Copyrighted",
            rvprop="content",
            rvslots="main",
            titles="|".join(titles),
        )
        pages = {page["title"]: page for page in result["query"]["pages"]}
        for title in titles:
            page = pages.get(title)
            if not page or "imageinfo" not in page:
                failures[title] = "missing"
                continue
            meta = page["imageinfo"][0]["extmetadata"]
            license_id = meta.get("License", {}).get("value")
            copyrighted = meta.get("Copyrighted", {}).get("value")
            wikitext = page["revisions"][0]["slots"]["main"]["content"]
            if license_id != "pd" or copyrighted != "False":
                failures[title] = f"License={license_id} Copyrighted={copyrighted}"
            elif not re.search(r"\{\{\s*PD-", wikitext, re.I):
                failures[title] = "no PD-* template"
    return failures


def check_nemo(update):
    if update:
        strips = nemo_listing()
        STRIPS.write_text("".join(f"{d}\t{strips[d]}\n" for d in sorted(strips)))
    rows = [line.split("\t") for line in STRIPS.read_text().splitlines()]
    failures = nemo_licence_failures([name for _, name in rows])
    by_year = collections.Counter(date[:4] for date, _ in rows)
    for year, count in sorted(by_year.items()):
        print(f"commons  Little Nemo {year}: {count} strips")
    for title, reason in failures.items():
        print(f"FAIL     {title}: {reason}")
    return not failures


def check_ndl():
    ok = True
    for work in json.loads(WORKS.read_text()):
        for volume in work["volumes"]:
            pid = volume["pid"]
            try:
                manifest = get_json(f"https://dl.ndl.go.jp/api/iiif/{pid}/manifest.json")
            except (OSError, http.client.HTTPException, ValueError) as error:
                print(f"FAIL     {pid} {work['title']}: {error}")
                ok = False
                continue
            metadata = {row["label"]: row["value"] for row in manifest.get("metadata", [])}
            access = metadata.get("Access Restrictions")
            frames = len(manifest["sequences"][0]["canvases"]) if "sequences" in manifest else 0
            status = "ndl     " if access == "PDM" and frames else "FAIL    "
            ok = ok and access == "PDM" and frames > 0
            print(
                f"{status} {pid} {manifest.get('label')}: {access}, {frames} frames, "
                f"{metadata.get('Creator')}, {metadata.get('Publication Date')}"
            )
            time.sleep(2)
    return ok


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--update", action="store_true", help="regenerate strips.tsv")
    args = parser.parse_args()
    nemo_ok = check_nemo(args.update)
    ndl_ok = check_ndl()
    sys.exit(0 if nemo_ok and ndl_ok else 1)


if __name__ == "__main__":
    main()
