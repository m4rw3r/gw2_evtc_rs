
## TODO

* [ ] Stats-summary
  * [x] Subgroup
  * [x] Profession
  * [ ] Weapons used?
  * [x] Character name
  * [x] Account name
  * [x] Point of View
  * [x] Boss DPS
    * [x] Physical
    * [x] Condi
  * [x] All DPS
    * [x] Physical
    * [x] Condi
  * [ ] Incoming total damage
  * [ ] Wasted time casting things
  * [ ] Failed mechanics
  * [x] Critical-rate, Scholar uptime (count hits), seaweed uptime (count hits)
  * [ ] Times downed
  * [ ] Dead with time (and percent of fight)
  * [ ] Summary for group and squad
* [ ] Boon- and buff-summary
  * [ ] Boon-uptime
  * [ ] Buff-uptime
  * [ ] Averages for subgroup/squad
* [ ] Player summary
  * [x] Skills used, grouped per agent
    * [x] Total damage
    * [ ] Damage percentage
    * [x] Number of hits, min, max, average, critical-rate, flanking-rate, glancing-rate, wasted-time
  * [x] Total Boss and cleave damage
  * [x] Critical rate, Glance rate, moving while hitting (seaweed uptime), Scholar uptime (per hits), missed hits, interrupted hits
    * [ ] casting time wasted
  * [ ] Average HP%, lowest HP%
  * [ ] Downed, death, dodges, evades, blocks
  * [ ] Damage taken, damage absorbed
  * [ ] Boons stripped
  * [ ] Absorbed by enemy?
  * [ ] Rotation
    * [ ] Info about quickness y/n on cast
    * [ ] Info about aborted casts
    * [ ] Info about boons on boon-sharing stuff which requires boons
* [ ] Graphs
  * [ ] Fulltime boss-dps
  * [ ] 10s DPS
* [x] JSON-output
* [ ] HTML-output with graphs
  * [ ] Decide on graph-library
  * [ ] Write HTML-skeleton
  * [ ] Write JS
* [ ] Condition uptime on boss for all conditions
  * [ ] average stacks
  * [ ] graph
* [ ] Colour lines based on core-profession
* [ ] Display used weapons for the character


## Notes

* Performance regression when refactoring to use a large Event-enum (improves correctness, ease of use). From 96822ed0e0cc62dceb5f6d8a8c6307aa628fb76f to f1eafc49e5eac84ceaad9b5725117ebaf42661bf, approx 30% performance loss (600ms -> 800ms).