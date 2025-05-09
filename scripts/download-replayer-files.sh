#!/bin/bash

# Display help menu
usage() {
    echo ""
    echo "download-replayer-files.sh"
    echo "=========================="
    echo ""
    echo "Given parameters specifying a date, download"
    echo "the whirlpool state and transaction files"
    echo "for that date."
    echo ""
    echo "Examples:"
    echo "# Downloading files for November 30, 2023":
    echo "./download-replayer-files.sh --year=2023 --date=1130"
    echo "./download-replayer-files.sh -y=2023 -d=1130"
    echo ""
    echo "# For quick staging, defaults to 20231130 when --year is not specified:"
    echo "# 2022 files are reasonable in size, 2023 is getting large (e.g., 180mb)."
    echo "./download-replayer-files.sh"
    echo ""
    echo "Notes:"
    echo "1. We use .gitignore in 2023 and 2024 folders to ignore the downloaded files."
    echo ""
    echo "Parameters:"
    echo "-y|--year   The year for the files to download"
    echo "-d|--date   The date for the files to download"
    echo ""
    echo "-h|--help   Print help for script"
    echo ""

    exit
}

# Parse input arguments
for i in "$@"
do
case $i in
    -h|--help)
    usage
    shift
    ;;
    -y=*|--year=*)
    YEAR="${i#*=}"
    shift
    ;;
    -d=*|--date=*)
    DATE="${i#*=}"
    shift
    ;;
    *)
    echo "Unknown option: $i"
    usage
    shift
    ;;
esac
done

# Validate input arguments and set defaults
if [[ "$YEAR" == "" ]]; then
    echo ""
    echo "======================="
    echo "Defaulting to 20231130."
    echo "======================="
    echo ""
    YEAR="2023"
    DATE="1130"
fi
if [[ "$DATE" == "" ]]; then
    echo "--date is mandatory"
    usage
fi

# Navigate to where we stage download files for the specified year
cd ../qtrade-replayer/whirlpool-tx-replayer/data/sample_local_storage/${YEAR}

# Clean up any previous download folder for the specified date
if [ -d ${DATE} ]; then rm -rf ${DATE}; fi

# Create/navigate to where we donwload files for the specified date
mkdir -p ${DATE}
cd ${DATE}

# Download the files for the specified date
curl -OL https://whirlpool-replay.pleiades.dev/alpha/${YEAR}/${DATE}/whirlpool-state-${YEAR}${DATE}.json.gz
curl -OL https://whirlpool-replay.pleiades.dev/alpha/${YEAR}/${DATE}/whirlpool-transaction-${YEAR}${DATE}.jsonl.gz
