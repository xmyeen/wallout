#!/bin/env Python

import ctypes
import os,sys

dll = ctypes.CDLL(sys.argv[1])

start_server_fn = dll.start_server
start_server_fn.argtypes = [ctypes.c_char_p]

configuration_file_path = bytes(sys.argv[2],encoding='utf-8')
start_server_fn(configuration_file_path)

# if __name__ == '__main__':
#     print('123')