# Data capture app for wii-ext-rs

In order to test and develop wii-ext-rs, it was useful to have samples of real data to develop against.  
This was a quick program I put together to generate the test data.

## Instructions
Grab a Raspberry Pi Pico  
Plug a Wii extension controller into one of the i2c ports (you can buy an adapter or cut an extension cable).  
Update `CONTROLLER_NAME` so that the output is specific to your controller name.  
Adjust the I2C port and pins to match.  
Run the program. Follow the prompts.  
Save the output to a text file (I'll use `output.txt`)  
run  
```console
grep -v '//' output.txt > output_no_comments.rs
```

Paste that data into `test_data.rs` in `wii-ext-rs`.  
Write some tests!