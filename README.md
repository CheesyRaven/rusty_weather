# rusty_weather

This is just a little app using the OpenWeatherMap API to create a little weather display in your terminal. The idea would be to have it set up to run when you start the terminal and get a cute little weather display with some simple ASCII art, but it also includes a feature to call against a specific zip code if you want to see weather somewhere else. 

# Usage

When starting, you can run `rusty_weather --setup` to be walked through adding your OpenWeatherMap API key as well as have the app lookup the coordinates for your zip code and save them for the weather calls. These are saved in a config.yaml file. 

Once configured, you can run without flags to get a display of the current weather for your configured area. If you want to see another area, just add the `--zip` flag with a zip code argument: `rusty_weather -- --zip 12345`

```
    .-.     | Temperature: 19.29
 .-(   ).   | Min: 20.21
(________)  | Max: 0
Schenectady | Wind Speed: 17.27
```
