const { get } = require("https");

const API_HOST = "api.guildwars2.com";
const API_BASE = "/v2";
const call     = url => new Promise((resolve, reject) => get({
  host: API_HOST,
  path: API_BASE + url,
}, res => {
  res.setEncoding("UTF-8");

  let data = "";

  res.on("data", d => data += d);
  res.on("end", () => {
    const parsed = JSON.parse(data);
    if(parsed) {
      resolve(parsed);
    }

    reject(data);
  });
}));

const chunk = (arr, chunkSize) => {
  let ret = [];

  for(let i = 0; i < arr.length; i += chunkSize) {
    ret.push(arr.slice(i, i + chunkSize));
  }

  return ret;
};

const querySkills = skillIds => call("/skills?ids=" + skillIds.join(","))
const flatten     = arrs => [].concat.apply([], arrs);
// TODO: Generate Rust code instead and let it include the data?
const writeData   = data => Object.keys(data)
                                  .forEach(k => process.stdout.write(`export const ${k} = ${JSON.stringify(data[k], null, 2)};
`));

call("/skills")
  .then(skills => Promise.all(chunk(skills, 200).map(querySkills)))
  .then(flatten)
  .then(skills => {
    /*
    const skillData = skills.map(({ id, name, icon, type, weapon_type, slot }) => ({
      id,
      name,
      icon,
      type,
      weapon_type,
      slot,
      */
    const weapon1Skills = skills.reduce((a, { id, slot }) => { if(slot === "Weapon_1") { a[id|0] = true } return a; }, {});
    const weaponType = skills.reduce((a, { id, weapon_type, slot }) => {
      if(weapon_type && weapon_type !== "None") {
        let hand = slot === "Weapon_4" || slot === "Weapon_5" ? "off" : "main";

        a[id|0] = {
          weapon: weapon_type,
          hand,
        };
      }

      return a;
    }, {});
    const skillNames = skills.reduce((a, { id, name }) => { a[id|0] = name; return a; }, {});
    const skillIcons = skills.reduce((a, { id, icon }) => { a[id|0] = icon; return a; }, {});
    const buffIcons  = flatten(skills.map(s => (s.facts || []).concat(s.traited_facts || [])))
      .reduce((a, { status, type, icon }) => {
        if((type === "Buff" || type === "PrefixedBuff") && icon && ! a[status]) {
          a[status] = icon;
        }

        return a;
      }, {});

    return {
      skillIcons,
      skillNames,
      weaponType,
      buffIcons,
      weapon1Skills,
    };
  })
  //.then(console.log);
  .then(writeData);