#!/usr/bin/python3
import random, string, os, subprocess, shutil, pdb
from pathlib import Path

def gen_string(path_len: int = 16) -> str:
	x = os.urandom(path_len >> 1)
	return x.hex()


def setup(path: str = "samplepath", maxpaths: int = 2**16, pathlen: int = 16) -> str:
	if os.path.exists(path):
		shutil.rmtree(path)

	os.makedirs(path)

	for i in range(maxpaths):
		os.mknod(f"samplepath/{gen_string()}")

	infile: str = f"{path}/infile.txt"

	findcmd = subprocess.Popen(["sh", "-c", f"find {infile}"], shell=False, stdout=subprocess.PIPE)
	with open(infile, "wb") as f:
		f.write(findcmd.stdout.read())
	
	return infile

def unsetup(path: str = "samplepath"):
	shutil.rmtree(path)

def main(maxpaths: int = 2**8):
	infile = setup(maxpaths=maxpaths)
	p = subprocess.Popen(["sh", "-c", f"hyperfine -N \"dockertarget/textperms --force-path -f {infile} -o /dev/null\""])
	p.wait()
	unsetup()

	

if __name__ == "__main__":
	maxpaths: int = 2**20
	main(maxpaths=maxpaths)