An implementation of the simplex algorithm by Antonin Lenz and Léo Stefanesco

Is is written in rust, here is how to install it:

To Install Rust
===============

Execute the following command, it's probably gonna be fine.

```bash
curl -s https://static.rust-lang.org/rustup.sh | sudo sh -s -- --channel=nightly
```

More info at rust-lang.org

The Simplex
===========

Usage :
```bash
toto [--bland] [--latex] file.lp
```

There are two heuristics for the choice of the entering variable:
Bland's rule (which terminates), and choosing the one with the 
greatest coefficient, which seems to be faster.

Note: In the PDF output, the name of the variables might not be the same
as those in the input program and x_0 is the constant factor.