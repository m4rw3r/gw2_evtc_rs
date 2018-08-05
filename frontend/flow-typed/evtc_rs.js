
type EncounterData = {
  boss:        string,
  gameBuild:   number,
  lang:        string,
  // End of log, unix timestamp in seconds
  logEnd:      number,
  logName:     string,
  // Start of log, unix timestamp in seconds
  logStart:    number,
  serverShard: number,
  succes:      bool,
};

type BuffData = {
  name:    string,
  skillId: number,
};

type AgentData = {
  accountName: string,
  concentration: number,
  conditionDmg:  number,
  // milliseconds
  diedAt:        number,
};

type EnemyData = {
  agent: AgentData,
};

type Data = {
  buffs:     { [skillId:number]: BuffData },
  encounter: EncounterData,
  enemies:   Array<EnemyData>,
};