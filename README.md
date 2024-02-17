# takeout-travel-history
Reads your Google Takeout data from maps to generate a list of each time you changed countries

# Usage

This program is designed to work with Google Takout data from Google Maps Timeline.


## Example Invocation

This example skips data gaps and subregion crossings, and excludes location data from unknown sources and wifi, as well as excluding location data that has no source. <br />
`./takeout-travel-history.exe border-crossings -p "C:\Users\XXX\Downloads\takeout-20240112T184310Z-001.zip" -s -m -e wifi -e none -e unknown` <br />

Here is an example of what the output might look like (this would be a trip from America to Japan with a 1-stop layover in Canada via YYZ):
```
...
Wed, 26 Jul 2023 22:30:28 +0000
    |
    | Ontario
    | Canada
    | Duration: 0 Days
    |
Thu, 27 Jul 2023 05:09:27 +0000
    |
    | Japan
    | Duration: 13 Days
    |
Thu, 10 Aug 2023 04:39:00 +0000
    |
    | Ontario
    | Canada
    | Duration: 0 Days
    |
Fri, 11 Aug 2023 02:48:45 +0000
    |
    | United States of America
    | New York
    | Duration: 156 Days
    |
...
```

## Fetching Your Timeline Data

To get your Timeline data, navigate to https://takeout.google.com/settings/takeout/

By default, all your data is selected for export; you can use the "Deselect all" button to ignore the data

<img alt="Choosing the Deselect all option" src="images/deselect-all.png" width=500 />

Then, scroll down and select "Location History (Timeline)" for export

<img alt="Selecting Location History for export" src="images/location-history.png" width=500 />

Select "Next step" to proceed

<img alt="Selecting Next step" src="images/next-step.png" width=500 />

Select your export options; ensure "`.zip`" is selected as the file type

<img alt="Selecting export options" src="images/export-options.png" width=500 />

Then, retrieve your `.zip` file and point this tool at it using the `-p` option. You can also extract the `Records.json` from the file and supply that path instead.

## Detailed Usage

Refer to the CLI's `--help` output for more details; reproduced here: `takeout-travel-history.exe --help`
```
Usage: takeout-travel-history.exe [COMMAND]

Commands:
  border-crossings  lists every time the location crosses a recognized border
  help              Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

For the border crossings command: `takeout-travel-history.exe --help`
```
lists every time the location crosses a recognized border

Usage: takeout-travel-history.exe border-crossings [OPTIONS] --path <PATH>

Options:
  -p, --path <PATH>              The .zip or .json file that will be read to produce the command's output
  -e, --exclude-source <SOURCE>  Excludes a certain data source from the results; can be specified multiple times to exclude multiple sources [possible values: wifi, gps, cell, unknown, none]
  -s, --ignore-subregions        Ignores border crossings between subregions such as US states, Canadian provinces, etc
  -m, --ignore-missing-data      Does not treat missing data as its own region and instead assumes that the region remains the same for the duration of missing data
  -h, --help                     Print help (see more with '--help')
```


# TODO

- make order of regions consistent (implement `Ord` for `Region`)
    - order by scale then lexicographically?