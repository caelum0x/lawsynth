"""Executable security checks for the public LawSynth bundle boundary."""
from __future__ import annotations
import hashlib, json, struct, subprocess, sys, tempfile, tomllib, zipfile
from pathlib import Path
from typing import Any
ROOT = Path(__file__).resolve().parents[2]
MANIFEST = b'{\n  "format": "lawsynth-world",\n  "format_version": "0.1.0",\n  "world_encoding": "lawsynth-world-binary-v1"\n}\n'
def native_cli(*arguments: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["cargo","run","--quiet","-p","lawsynth-cli","--bin","lawsynth","--",*arguments],cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
def string(value: str) -> bytes:
    data=value.encode("utf-8"); return struct.pack("<H",len(data))+data
def valid_world() -> bytes:
    return b"LSW1"+struct.pack("<I",1)+string("x")+b"\x00\x00"+struct.pack("<I",0)+struct.pack("<I",1)+string("x")+b"\x00"+struct.pack("<d",1.0)
def deep_world() -> bytes:
    expression=b"\x00"+struct.pack("<d",1.0)
    for _ in range(128): expression=b"\x02\x00"+expression
    return b"LSW1"+struct.pack("<I",1)+string("x")+b"\x00\x00"+struct.pack("<I",0)+struct.pack("<I",1)+string("x")+expression
def write_bundle(path: Path, world: bytes, *, names: tuple[str,str,str]=("manifest.json","world/world.bin","provenance/checksums.sha256"), compression: int=zipfile.ZIP_STORED) -> None:
    core={names[0]:MANIFEST,names[1]:world}
    core[names[2]]="".join(f"{hashlib.sha256(data).hexdigest()}  {name}\n" for name,data in core.items()).encode("ascii")
    with zipfile.ZipFile(path,"w",compression=compression) as archive:
        for name,data in core.items(): archive.writestr(name,data)
def load(directory: Path) -> tuple[dict[str,Any],dict[str,Any],dict[str,Any]]:
    with (directory/"case.toml").open("rb") as stream: case=tomllib.load(stream)
    input_data=json.loads((directory/"input.json").read_text(encoding="utf-8"))
    expected=json.loads((directory/"expected.json").read_text(encoding="utf-8"))
    assert case["case"]["id"]==directory.name==input_data["case_id"]==expected["case_id"]
    return case,input_data,expected
def assert_case(directory: Path) -> None:
    case,input_data,expected=load(directory); kind=case["case"]["kind"]
    with tempfile.TemporaryDirectory(prefix=f"lawsynth-security-{directory.name}-") as temporary:
        bundle=Path(temporary)/"fixture.lsworld"
        if kind=="archive-traversal": write_bundle(bundle,valid_world(),names=("manifest.json","../world/world.bin","provenance/checksums.sha256")); result=native_cli("inspect",str(bundle))
        elif kind=="decompression-limits": write_bundle(bundle,valid_world(),compression=zipfile.ZIP_DEFLATED); result=native_cli("inspect",str(bundle))
        elif kind=="expression-limits": write_bundle(bundle,deep_world()); result=native_cli("inspect",str(bundle))
        else: result=native_cli(input_data["command"])
        combined=result.stdout+result.stderr
        assert result.returncode != 0, "security boundary unexpectedly accepted the input"
        assert expected["error_contains"] in combined,combined
    print(f"{directory.name}: native security boundary rejected as documented")
if __name__=="__main__": assert_case(Path(sys.argv[1]).resolve())
