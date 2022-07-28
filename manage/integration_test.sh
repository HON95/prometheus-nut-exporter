#!/bin/bash

set -u -o pipefail

manage/nut-server-mock.py &>/dev/null &
mock_pid=$!

cargo run &>/dev/null &
exporter_pid=$!

sleep 1

scrape_content=$(curl -sSf "http://localhost:9995/nut?target=localhost:3493")

kill -9 $mock_pid &>/dev/null
kill -9 $exporter_pid &>/dev/null

if ! echo $scrape_content | grep 'nut_status{ups="alpha"} 1' &>/dev/null; then
    echo "Failure!"
    exit 1
fi

echo "Success!"
