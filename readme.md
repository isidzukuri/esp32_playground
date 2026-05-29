build and flash to esp32
```
cargo espflash flash --monitor
```


web page thru wifi
- server should run in paralel thread
- design some kind of agregation
- plot per 1m / 10m / 1h / 8h / 1d / 
save log to sd
- estimate log size and design some kind of log rotation
- concurent access to log
attach temperature/humidity sensor
some kind of log map/reduce. Maybe save some aggregated value
save logs as csv
log writer append new line in the end of log
write log if volume above trashhold
1 log per second. If at least one change was detected, than this second must be stored to log
store log only if change detected othervise write nothing to save storage. On client side generate values in between logs and prepare for the plot

log format:
```
timestamp,temperature,humidity,volume
2025-08-25T12:00:00,24.1,45.2,1500
```


The Architecture Strategy

    Thread 1 (Data Producer): High-priority. Reads the sensor every 30ms. It doesn't care about anything else.

    Thread 2 (Storage Manager): Medium-priority. Receives data from Thread 1 and writes it to the SD card.

    Thread 3 (Web Server/Consumer): Low-priority. When a user visits the webpage, it reads from the SD card and sends the data over HTTP.

Yes. The ESP32 is a dual-core processor.

    web server and log reader to core 0 (PRO_CPU) (the Wi-Fi/Radio core)

    sensor reader and log writer goes to core 1 (APP_CPU) 

Pro-tip: Since you are writing to the SD card on Core 1, ensure your SD card card reader pins are physically routed to the pins assigned to the SPI peripheral you initialize in your code.


dht delay must be 2 seconds at least