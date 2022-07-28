#!/bin/bash

set -u -o pipefail

# Start mock
manage/nut-server-mock.py &>/dev/null &
mock_pid=$!

# Start exporter
cargo run &>/dev/null &
exporter_pid=$!

# Teardown on exit
function teardown {
    kill -9 $mock_pid &>/dev/null
    kill -9 $exporter_pid &>/dev/null
}
trap teardown EXIT

# Wait for startup
sleep 1

# Scrape
scrape_content=$(curl -sSf "http://localhost:99951/nut?target=localhost:3493")
if [[ $? != 0 ]]; then
    echo "Failure!"
    echo "Scrape failed."
    exit 1
fi

# Validate result
if ! echo $scrape_content | grep 'nut_status{ups="alpha"} 1' &>/dev/null; then
    echo "Failure!"
    echo "Scraped result contains errors."
    exit 1
fi

echo "Success!"
