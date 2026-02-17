# Lococode
Location codes inspired by Google's Plus Codes.

## Usage
### Encode
Following the example cited on the [Open Location Code's wikipedia page](https://en.wikipedia.org/wiki/Open_Location_Code) for the Merlion fountain in Singapore located at:

`1.286785°N 103.854503°E`

```bash
$ lococode encode --lat 1.286785 --lon 103.854503
WXLK
```

You can specify the desired code length with the `-l N` flag:

```bash
$ lococode encode --lat 1.286785 --lon 103.854503 -l8
WXLKNTI5
```

Notice lococode does not use padding. Perhaps it should? Please open an issue if you think so.

### Decode
```bash
$ lococode decode WXLKNTI5
- Center.......... 1.286860, 103.854618
- Radial bounds... ±0.000086, ±0.000172
- Metric bounds... ±9.491, ±18.977
- Error radius.... 21.218
```

### Test
Same as encoding and then decoding, with additional metrics:

```bash
$ lococode test --lat 1.286785 --lon 103.854503 -l6
In..... 1.286785, 103.854503
Code... WXLKNT
Out
- Center.......... 1.288147, 103.859253
- Radial bounds... ±0.002747, ±0.005493
- Metric bounds... ±303.703, ±607.252
- Error radius.... 678.963
Latitude error.... 0.001361973
Longitude error... 0.004749930
Encoding time..... 820ns
Decoding time..... 7.95µs
Round trip time... 8.77µs
```

## Design
- Base32
- Alternates between 2-3 bits for latitude and longitude
