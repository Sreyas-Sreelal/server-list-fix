# Server List Fix

This is a samp client server list fix, which reroutes the client's request to `list.sa-mp.com` to `sam.markski.ar`. The idea is originally from [spmn](https://github.com/spmn/sa-mp_masterlist_fix). I just wrote this in Rust for intellectual curiosity. If you're someone who looking to do similar things in Rust, this code base might be useful for you. I tried to keep the implementation really basic, and if you're looking for an actual fix instead of learning how to do this in Rust, original one by spmn is probably more suited for you. I have also added comments to the code, explaining my thought process while writing this code.

Incase you want to use it

Build the code using nightly toolchain

```
cargo +nightly build --release 
```

place the built `version.dll` in your GTA_SA game directory
