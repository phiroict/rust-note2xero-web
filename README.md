# Noted2Xero web application   
REST interface for the Noted2Xero core library 

## Web
Needs the nightly build of the Rust stack, (make init will set this up for you) This runs the CSV converter as a web service, its entrypoint is

rustup override set nightly
Run the component by

```bash
./noted2xero_web
```

Check out the make run_web target for more context.

The web component will run until killed listening on port 8000 it expects a POST to http://YourHostHere:8000/noted with a form with data as the Noted CSV payload and text as the invoice number. An example web page is at
