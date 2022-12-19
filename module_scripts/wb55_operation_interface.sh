#!/bin/bash

#Help
if [[ "$1" == "-h" ]]
then
  printf "Description: This script can install the required program on the stm32wb55rg, read the memory or reset the device.
  Usage: This script expect 2 arguments: command, [input_file], where:
   \t command = program to program a binary to the memory, compare to read a binary from memory and compare with the file passed, reset to reset the device
   \t file = Input file if program (necessary be a binary (.bin)), output file if read (if not passed,  will read the serial id from board)\n"
  exit 0
fi

#Warning about arguments
if [[ $# -eq 0 ]]
then
    echo "command expected"
    exit 0
fi

if [[ $1 == *"program"* ]] && [[ $# -ne 2 ]]
then
    echo "binary file expected"
    exit 0
fi

bin_path=""
if [[ $# -eq 2 ]]
then
    bin_path=$2
fi

sudo pkill openocd
sleep 1

#execute script
if [[ $1 == *"reset"* ]]
then
    timeout 30 ./wb55_operation.sh $1 > file 2>&1
elif [[ $1 == *"read"* ]] && [[ $bin_path == "" ]] 
then
    timeout 60 ./wb55_get_board_serial.sh > file 2>&1
else
    timeout 120 ./wb55_operation.sh $1 $bin_path > file 2>&1
fi

error=false
while read line;
do
    if [[ $line == *"Connection refused"* ]]
    then
        error=true
        break;
    elif [[ $line == *"accepting 'telnet' connection"* ]]
    then
        break;
    fi
done < file

if test -f "result" && [[ $error = false ]]
then
    cat result
else
    echo "Device not found"
fi
