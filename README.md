# Use Google location history to figure out where you've been

## Getting started

go to https://www.google.com/takeout then click "select none", then scroll down and click the box next to "Location History  JSON format", scroll down and select next, then create archive. Then go to https://github.com/etrombly/country_parser/releases and download the country_parser or country_parser.exe (if you're using windows). Unzip your location history and make sure it is in the same location as country_parser, open a terminal and run country_parser. It should output a list of dates and countries visited. I noticed a few entries in my location history that were wrong, and if your phone is on while traveling it will show countries that you were just in the airport. Should help you out with dates though.