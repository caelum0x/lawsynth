"""Executable compatibility checks for the current public LawSynth format."""
from __future__ import annotations
import hashlib,json,struct,subprocess,sys,tempfile,tomllib,zipfile
from pathlib import Path
from typing import Any
ROOT=Path(__file__).resolve().parents[2]
CURRENT_MANIFEST=b'{\n  "format": "lawsynth-world",\n  "format_version": "0.1.0",\n  "world_encoding": "lawsynth-world-binary-v1"\n}\n'
def native_cli(*arguments: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["cargo","run","--quiet","-p","lawsynth-cli","--bin","lawsynth","--",*arguments],cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
def string(value: str)->bytes:
    data=value.encode("utf-8"); return struct.pack("<H",len(data))+data
def world()->bytes:
    return b"LSW1"+struct.pack("<I",1)+string("x")+b"\x00\x00"+struct.pack("<I",0)+struct.pack("<I",1)+string("x")+b"\x00"+struct.pack("<d",1.0)
def write_bundle(path:Path,manifest:bytes)->None:
    entries={"manifest.json":manifest,"world/world.bin":world()}
    entries["provenance/checksums.sha256"]="".join(f"{hashlib.sha256(value).hexdigest()}  {name}\n" for name,value in entries.items()).encode("ascii")
    with zipfile.ZipFile(path,"w",compression=zipfile.ZIP_STORED) as archive:
        for name,value in entries.items(): archive.writestr(name,value)
def load(directory:Path)->tuple[dict[str,Any],dict[str,Any],dict[str,Any]]:
    with (directory/"case.toml").open("rb") as stream: case=tomllib.load(stream)
    input_data=json.loads((directory/"input.json").read_text(encoding="utf-8")); expected=json.loads((directory/"expected.json").read_text(encoding="utf-8"))
    assert case["case"]["id"]==directory.name==input_data["case_id"]==expected["case_id"]
    return case,input_data,expected
def assert_case(directory:Path)->None:
    case,input_data,expected=load(directory)
    if case["case"]["kind"]=="plugin-protocol": result=native_cli(input_data["command"])
    else:
        with tempfile.TemporaryDirectory(prefix=f"lawsynth-compat-{directory.name}-") as temporary:
            bundle=Path(temporary)/"fixture.lsworld"; write_bundle(bundle,input_data["manifest"].encode("utf-8")); result=native_cli("inspect",str(bundle))
    text=result.stdout+result.stderr
    assert result.returncode != 0,"unsupported compatibility input unexpectedly loaded"
    assert expected["error_contains"] in text,text
    print(f"{directory.name}: compatibility boundary is explicit")
if __name__=="__main__": assert_case(Path(sys.argv[1]).resolve())
