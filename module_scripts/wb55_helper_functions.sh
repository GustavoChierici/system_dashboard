#!/bin/bash

#Function to compare two string
#params: string
#        matchs (can be an array separeted with ',')
compare_string() {
    main_string=${1,,}
    matchs=${2}
    readarray -d , -t strarr <<< "$matchs"
    for (( n=0; n < ${#strarr[*]}; n++))
    do
        word=$(echo ${strarr[n]} |tr -d '\n')
        word=${word,,}
        if [[ $main_string == *"$word"* ]]
        then
            return 1
        fi
    done
    return 0
}

#Function to read the debug file until find the expected string or a timeout occur
#params: first_match
#        second_match
#        timeout in seconds (not requerid)
current_line=0
read_file() {
    i=1;
    [[ $# -eq 3 ]] && max_timeout=$3 || max_timeout=120
    for attempts in $(seq 1 $max_timeout);
    do
        tail -n +${current_line} debug | while read -r line;
        do
            found=0 
            compare_string "$line" "$1"
            found=$?
            if [ $found -eq 0 ]
            then
                compare_string "$line" "$2"
                found=$?
            fi
            ((i++))
            echo $i > current_index_file
            echo $line > current_line_file
            if [ $found -eq 1 ]
            then
                echo "found" > found_result
                break;
            fi            
        done
        if test -f "found_result"; then
            sudo rm found_result
            break;
        fi
        sleep 1
    done
    current_line=$(( current_line + $(cat current_index_file) ))
    line=$(cat current_line_file)
}

#Function to verify if there's an error
#params: first_match - not required
#        error_msg
#        exit_on_error - not required, default 1, but if passed need to be with all 3 args
error=0
handle_error() 
{
    [[ $# -ge 2 ]] && error_desc=$2 || error_desc=$1
    [[ $# -eq 3 ]] && exit_on_error=$3 || exit_on_error=1
    if [ $# -ge 2 ] && [ ${#line} -ne 0 ]
    then
        compare_string "$line" "$1"
        found=$?
        if [ $found -eq 1 ]
        then
            error=1
            if [[ $exit_on_error -eq 1 ]]
            then
                echo "Error: $error_desc" >> result
                echo "exit"
                sudo pkill openocd
            fi
        fi
    else
        error=1
        echo "Error: $error_desc" >> result
        echo "exit"
        sudo pkill openocd
    fi
}

#Function to run a command
#params: command
#        first match to read debug
#        second match to read debug
#        first match error (not necessary)
#        error message
#        max_timeout (not necessary buf if passed necessary be with all others args)
execute_command()
{
    command_attempts=10
    command=$1
    first_match=$2
    second_match=$3
    [[ $# -ge 5 ]] && error_desc=$5 || error_desc=$4
    [[ $# -eq 6 ]] && timeout=$6 || timeout=60
    while [ $command_attempts -gt 0 ];
    do
        error=0
        echo $command
        sleep 1
        read_file "$first_match" "$second_match" "$timeout"
        [[ $# -eq 5 ]] && handle_error "$4" "$error_desc" 0 || handle_error "$error_desc"
        if [[ error -eq 0 ]]
        then
            command_attempts=0
        else
            ((command_attempts--))
        fi  
    done
    [[ $# -eq 5 ]] && handle_error "$4" "$error_desc" 1 || handle_error "$error_desc"
}

