#!/usr/bin/env python3

with open("../auth_aai.txt", "r") as f:
    auth_key = f.read().strip()

if __name__ == "__main__":
    print(auth_key)
