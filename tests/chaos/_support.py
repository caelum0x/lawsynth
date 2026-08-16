"""Executable local chaos-resilience checks for the stateless LawSynth CLI."""
from __future__ import annotations
import hashlib,json,struct,subprocess,sys,tempfile,tomllib,zipfile
from pathlib import Path
from typing import Any
ROOT=Path(__file__).resolve().parents[2]
MANIFEST=b'{\n  "format": "lawsynth-world",\n  "format_version": "0.1.0",\n  "world_encoding": "lawsynth-world-binary-v1"\n}\n'
def native_cli(*arguments: str)->subprocess.CompletedProcess[str]:
    return subprocess.run(["cargo","run","--quiet","-p","lawsynth-cli","--bin","lawsynth","--",*arguments],cwd=ROOT,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
def string(value:str)->bytes:
    data=value.encode("utf-8");return struct.pack("<H",len(data))+data
def write_world_bundle(path:Path)->None:
    world=b"LSW1"+struct.pack("<I",1)+string("x")+b"\x00\x00"+struct.pack("<I",0)+struct.pack("<I",1)+string("x")+b"\x00"+struct.pack("<d",1.0)
    entries={"manifest.json":MANIFEST,"world/world.bin":world}
    entries["provenance/checksums.sha256"]="".join(f"{hashlib.sha256(value).hexdigest()}  {name}\n" for name,value in entries.items()).encode("ascii")
    with zipfile.ZipFile(path,"w",compression=zipfile.ZIP_STORED) as archive:
        for name,value in entries.items():archive.writestr(name,value)
def load(directory:Path)->tuple[dict[str,Any],dict[str,Any],dict[str,Any]]:
    with (directory/"case.toml").open("rb") as stream:case=tomllib.load(stream)
    input_data=json.loads((directory/"input.json").read_text(encoding="utf-8"));expected=json.loads((directory/"expected.json").read_text(encoding="utf-8"))
    assert case["case"]["id"]==directory.name==input_data["case_id"]==expected["case_id"]
    return case,input_data,expected
def assert_case(directory:Path)->None:
    case,input_data,expected=load(directory);kind=case["case"]["kind"]
    if kind in {"api-restart","scheduler-restart","worker-loss"}:
        result=native_cli(input_data["command"]);assert result.returncode != 0;assert expected["error_contains"] in result.stdout+result.stderr
        print(f"{directory.name}: service-plane capability is explicitly unavailable");return
    with tempfile.TemporaryDirectory(prefix=f"lawsynth-chaos-{directory.name}-") as temporary:
        bundle=Path(temporary)/"fixture.lsworld";write_world_bundle(bundle)
        first=native_cli("simulate",str(bundle),"--initial","x=0","--start","0","--end","1","--step","0.5")
        if kind=="storage-timeout":
            bundle.unlink();second=native_cli("simulate",str(bundle),"--initial","x=0","--start","0","--end","1","--step","0.5")
            assert first.returncode==0,first.stderr;assert second.returncode != 0;assert expected["error_contains"] in second.stdout+second.stderr
        else:
            second=native_cli("simulate",str(bundle),"--initial","x=0","--start","0","--end","1","--step","0.5")
            assert first.returncode==second.returncode==0,first.stderr+second.stderr;assert first.stdout==second.stdout,"duplicate stateless simulation was not deterministic";assert expected["stdout_contains"] in first.stdout
    print(f"{directory.name}: native deterministic/rejection behavior verified")
if __name__=="__main__":assert_case(Path(sys.argv[1]).resolve())
