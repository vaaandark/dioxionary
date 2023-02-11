#!/usr/bin/env lua

-- Usage: ./idx.lua path/to/idx
--        or version 3.0.0
--        ./idx.lua path/to/idx -3

local f = arg[1]
local pattern = arg[2] == '-3' and ">zI8I8" or ">zI4I4"

f = assert(io.open(f, "rb"))
local contents = f:read("a")
local now
local word, offset, size
while not now or now < #contents do
  word, offset, size, now = string.unpack(pattern, contents, now)
  print(string.format("%-50s | %8d | %4d", word, offset, size))
end

f:close()
