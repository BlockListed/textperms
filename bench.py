#!/usr/bin/python3
import random, string, os, subprocess, shutil, pdb
from pathlib import Path

maxpaths: int = 2**22

path: str = "samplepath"

path_len: int = 16

def gen_string():
	return ''.join(random.choice(string.hexdigits) for i in range(path_len))

if os.path.exists(path):
	shutil.rmtree(path)

os.makedirs(path)

for i in range(maxpaths):
	os.mknod(f"samplepath/{gen_string()}")

infile: str = f"{path}/infile.txt"

findcmd = subprocess.Popen(["sh", "-c", f"find {infile}"], shell=False, stdout=subprocess.PIPE)
with open(infile, "wb") as f:
	f.write(findcmd.stdout.read())

p = subprocess.Popen(["sh", "-c", f"hyperfine -N \"dockertarget/textperms --force-path -f {infile} -o /dev/null\""])
p.wait()

shutil.rmtree(path)