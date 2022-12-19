#!/bin/bash

#Function to program the options bits nBOOT0=1, nBOOT1=1 and nSWBOOT0=0
program_option_bytes()
{
    echo "stm32wbx option_write 0 0x20 0x8800000 0xC800000"
    sleep 0.1
    echo "stm32wbx option_write 0 0x14 0x8000000 0x8000000"
    sleep 1
    echo "reset halt"
    sleep 1
    echo "reset halt"
    read_file xPSR not
    handle_error not "Unable to halt"
    echo "init"
    sleep 0.1
    echo "stm32wbx option_read 0 0x20"
    sleep 0.5
    output=$(cat debug | grep -a '0x58004020' | tail -1)
    value=$(cut -d "=" -f2 <<< $output)
    value="${value:1:${#value}-2}"
    mask=0xC800000
    result=$(( $value & $mask ))
    if [[ $result -ne 0x8800000 ]]
    then
        handle_error
    fi
}

#Help
if [[ "$1" == "-h" ]]
then
  printf "Description: This script will install the required firmware on the stm32wb55rg. The output will be in the result file
  Usage: This script expect 3 arguments: firmware_path, bootloader_path and is_ble, where:
   \t firmware_path = path to the firmware. Tested with fus and ble stack 
   \t bootloader_path = path to the bootloader that is used to program the firmware 
   \t is_ble = 1 if the firmware is ble stack, 0 if not\n"
  exit 0
fi

#Warning about arguments
if [[ $# -ne 3 ]]
then
    echo "firmware path, bootloader path and ble are expected"
    exit 0
fi

#Set variables
sudo rm result
sudo rm debug
. ./wb55_helper_functions.sh --source-only
is_ble=$3
firmware_path=$1
bootloader_path=$2

if [[ $is_ble -eq 1 ]]
then
    address_write="0x080E1000"
    offset_compare="0xE1000"
    expected_result="00000601"
    # expected_protect_address="ca"
    expected_protect_address="e1"
else
    address_write="0x080EC000"
    offset_compare="0xEC000"
    expected_result="00000101"
    expected_protect_address="f4"
fi

#main function
sudo openocd & (sleep 1 &&
(
    #sleep 1
    #echo "reset_config connect_deassert_srst"
    sleep 1
    execute_command "reset halt" "xpsr" "not" "not" "Unable to halt"
    echo "init"
    sleep 0.1

    #First verify if it's necessary change option bytes
    echo "stm32wbx option_read 0 0x20"
    sleep 0.5
    output=$(cat debug | tr -d '\0' | grep -a '0x58004020' | tail -1)
    value=$(cut -d "=" -f2 <<< $output)
    value="${value:1:${#value}-2}"
    mask=0xC800000
    result=$(( $value & $mask ))
    if [[ $result -ne 0x8800000 ]]
    then
        program_option_bytes
    fi

    #Program bootloader
    execute_command "flash write_image erase unlock $bootloader_path 0x08000000" "wrote" "error,failed" "error,failed" "Unable to write"
    execute_command "flash verify_bank 0 $bootloader_path" "match" "error,differ" "error,differ" "Content from memory differs"
    echo "mww 0x20010004 1"
    sleep 0.5
    echo "mww 0x20010000 2"
    sleep 0.5
    echo "reset run"
    sleep 2
    own_reset_attempts=2
    while :
    do
        read_file breakpoints speed 3
        if [[ $line == *"breakpoints"* ]] || [[ $line == *"speed"* ]]
        then
            break;
        fi
        ((own_reset_attempts--))
        if [[ $own_reset_attempts -eq 0 ]]
        then
            break;
        fi
    done
    echo "reset halt"
    sleep 1
    execute_command "reset halt" "xpsr" "not" "not" "Unable to halt"
    echo "init"
    sleep 0.1

    #Verify bootloader execution
    echo "mdw 0x20010004"
    sleep 0.5
    output=$(cat debug | tr -d '\0' | grep -a '0x20010004' | tail -1)
    if [[ $output != *"00000102"* ]]
    then
        handle_error "Problem on delete stack"
    fi

    #Program firmware
    execute_command "flash write_image erase unlock $firmware_path $address_write" "wrote" "error,failed" "error,failed" "Unable to write"
    execute_command "flash verify_bank 0 $firmware_path $offset_compare" "match" "error,differ" "error,differ" "Content from memory differs"
    echo "mww 0x20010004 1"
    sleep 0.5
    echo "mww 0x20010000 4"
    sleep 0.5
    echo "reset run"
    sleep 10
    own_reset_attempts=2
    while :
    do
        read_file breakpoints speed 3
        if [[ $line == *"breakpoints"* ]] || [[ $line == *"speed"* ]]
        then
            break;
        fi
        ((own_reset_attempts--))
        if [[ $own_reset_attempts -eq 0 ]]
        then
            break;
        fi
    done
    echo "reset halt"
    sleep 1
    execute_command "reset halt" "xpsr" "not" "not" "Unable to halt"
    echo "init"
    sleep 0.1

    #Verify firmware instalation
    echo "mdw 0x20010004"
    sleep 0.5
    output=$(cat debug | tr -d '\0' | grep -a '0x20010004' | tail -1)
    if [[ $output != *$expected_result* ]]
    then
        handle_error "Problem on update firmware"
    fi
    echo "stm32wbx option_read 0 0x80"
    sleep 0.1
    output=$(cat debug | tr -d '\0' | grep -a '0x58004080' | tail -1)
    if [[ $output != *$expected_protect_address* ]]
    then
        handle_error "Problem on update firmware"
    fi

    #Reset and exit
    echo "reset"
    sleep 1
    echo "Success" >> result
    echo "exit"
) |  telnet localhost 4444  > debug) 
sudo pkill openocd

