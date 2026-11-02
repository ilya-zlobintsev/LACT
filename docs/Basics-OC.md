# Basics - Overclocking 
Methods described here are based on [fixme: link to basic.md]
It's not necessarily the only or optimal approach, but a solid starting point

## General info
### Memory offset
Memory clock/offset values might be a bit confusing. This is because each tool/website and even the driver itself might use different multipliers for VRAM clocks. The multipliers are different for different GPUs. Example:
GPU 4070 Super:
3dmark shows 1500-1600 Mhz using an x1 multiplier
LACT stat shows 10500 Mhz using an x8 multiplier, normalized to 1312
LACT offset for this GPU uses an x16 multiplier, meaning I need to set it to 3000 to reach 12000Mhz (x8), 1500 (x1)

Nvidia uses different multipliers for different interfaces. Because it's hard to test every hardware combination, lact doesn't try to translate them.
### Tools
- Furmark is great for testing cooling capabilities. Its use for testing stability is limited. You will most likely hit the power limit. It's fine.
- https://www.3dmark.com/search can provide a decent reference point for GPU capabilities. 

## Preparation
1. pick a game/benchmark you'll use during the process // fixme: too obvious?
2. write down the default voltage/clocks your GPU uses under load

## I want maximum performance (Overclock)

1. Set the power limit to the maximum
2. **Nvidia** Voltage Boost to 100%
3. start the game/benchmark
4. set the core offset to reach the reference value from 3dmark: 150-200Mhz (it's hard to provide a generic absolute value, so pick something realistic)
5. increase clocks until you see artifacts, then go back by 2-3 steps. Don't be too greedy. A crash in the middle of the game isn't worth 0.1% of performance
6. At this point I recommend using the profile for some time, so if it crashes, you know that it's the core offset that causes instability
7. Now that you're sure that the core clock is very stable, it's time to set the memory offset in basically the same way. Check 3dmark for reference, start the game/benchmark, increase the offset bit by bit until you see artifacts and go back by multiple steps

## I want a cool and quiet system (Undervolt)
the core principle for this method is to increase the power efficiency of the card by preventing it from reaching higher voltage points. There are multiple methods to do so

### Recommended - Locked clocks
with this method, we want to overclock first to find a stable core offset, then push gpu back to the voltage point we want
It's recommended because it uses the official public Nvidia api
1. start a benchmark/game with a stable load
2. **if core offset is available**, set the core offset to reach the reference value from 3dmark: 150-200Mhz (it's hard to provide a generic absolute value, so pick something realistic)
3. **if voltage offset is available and core offset is not**, reduce voltage by 5mV until you experience artifacts/instability, then go 2-3 steps back
4. **Nvidia** on the OC page, enable Locked clocks
5. decrease the maximum clock value until you reach the desired voltage

### Nvidia V/F Curve
The idea is to flatten the curve at a certain voltage point. If the GPU sees that there is no next voltage point with higher clocks, there is no need to increase voltage
1. click Custom VF Curve
2. open the VF curve
3. find a desired voltage point, right-click, then click flatten
4. now hold shift and drag the curve up. This will add a positive clock offset for the points

## I saw a tutorial which uses Afterburner and I want to replicate it
### Nvidia
Generally it should work. The only known difference is that you have to double the memory clock offset

When you copy a V/F Curve:
1. many cards have stock OC applied, meaning you cannot copy the offset directly
2. the V/F Curve can shift up and down based on the current temperature. When you want to achieve a certain target, e.g. 2750 Mhz / 930mV, you should start a benchmark and let the temp settle (to provide a stable temp). Flatten at 930, and adjust the offset until you reach the desired temp
