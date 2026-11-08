"""Download the voice models into the Hugging Face cache, each at its pinned
revision and only the files it needs (tools/voice/models.toml), and keep each
synthesis model's licence and card at the revision used under
assets/src/voice/models/<id>/.

    cargo xtask voice fetch [--only ID,...]

Nothing is sent but the requests for the files: no token, no account, no
personal data in a header.
"""

import argparse
import json
import pathlib
import sys
import tomllib
import urllib.request

from huggingface_hub import hf_hub_download, snapshot_download

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent.parent


def models():
    return tomllib.loads((HERE / "models.toml").read_text())["model"]


def card(repo, revision, name):
    """The file `name` of `repo` at `revision` from the model's own repository,
    or None when it has none"""
    try:
        return pathlib.Path(hf_hub_download(repo, name, revision=revision)).read_text(errors="replace")
    except Exception:
        return None


def upstream_sha(repo, short):
    """The full revision of `repo` that begins with `short`: the API's current
    one if it matches, else `short` as given (the file download resolves it)"""
    try:
        with urllib.request.urlopen(f"https://huggingface.co/api/models/{repo}", timeout=30) as r:
            sha = json.load(r).get("sha", "")
        return sha if sha.startswith(short) else short
    except Exception:
        return short


def keep_licence(m):
    """The upstream model's card and licence at its pinned revision, kept with
    the sources (what the game ships was made under these terms)"""
    out = ROOT / "assets/src/voice/models" / m["id"]
    out.mkdir(parents=True, exist_ok=True)
    rev = upstream_sha(m["upstream"], m["upstream_revision"])
    kept = []
    for name in ["LICENSE", "LICENSE.txt", "LICENSE.md", "README.md"]:
        text = card(m["upstream"], rev, name)
        if text is not None:
            (out / name).write_text(text)
            kept.append(name)
    (out / "SOURCE.toml").write_text(
        f'upstream = "{m["upstream"]}"\nupstream_revision = "{rev}"\n'
        f'repo = "{m["repo"]}"\nrevision = "{m["revision"]}"\nlicence = "{m["licence"]}"\n'
        f"kept = {json.dumps(kept)}\n"
    )
    return kept


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--only", default="", help="comma-separated model ids")
    a = ap.parse_args()
    only = {s for s in a.only.split(",") if s}
    failed = []
    for m in models():
        if only and m["id"] not in only:
            continue
        print(f"{m['id']}: {m['repo']}@{m['revision'][:12]}", flush=True)
        try:
            path = snapshot_download(m["repo"], revision=m["revision"], allow_patterns=m["include"])
            size = sum(f.stat().st_size for f in pathlib.Path(path).rglob("*") if f.is_file())
            print(f"  {size / 1e9:.2f} GB at {path}", flush=True)
            if m["role"] == "synth":
                print(f"  kept {', '.join(keep_licence(m)) or 'nothing'} of {m['upstream']}", flush=True)
        except Exception as e:
            print(f"  FAILED: {e}", flush=True)
            failed.append(m["id"])
    if failed:
        sys.exit(f"could not fetch: {', '.join(failed)}")


if __name__ == "__main__":
    main()
