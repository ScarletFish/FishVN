// 角色占位数据：取自 vndb-sample/characters-with-cv.json，字段形态对齐 /kana/char 接口。
// traits 只保留 spoiler: 0 且非成人内容，接口落地后沿用同一过滤。
// 元数据就位后整个文件删除，组件改为消费接口数据。
export const SAMPLE_CHARACTERS = [
  {
    "id": "c15",
    "name": "Emiya Shirou",
    "original": "衛宮 士郎",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/17/17.jpg"
    },
    "description": "Shirou is a good-hearted and honest teenager who always enjoys helping others. He is raised by [url=/c96619]Emiya Kiritsugu[/url], who imparts to him fundamental sorcery and \"The Ally of Justice\" ideal. He is in his second year of High School, and his hobbies include fixing various broken things, from VCRs to stoves, as well as cooking and cleaning.\n\nShirou is unskilled in most traditional forms of sorcery. He only knows Reinforcement, the power to analyze objects' structural composition to increase their effectiveness, such as the sharpness and durability of a sword and shield.\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Red",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sweater",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trousers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Dress Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Shinai",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Proactive",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Donkan",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Hard Worker",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Magician",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Swordsmanship",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Hero",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Orphan",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Cooking",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Assault",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Amnesia",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Hotblooded",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Amber",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Suit",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sports Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Ore",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Planning",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Friend",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Ward",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Cleaning",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Injury",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Archery",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Altruistic",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Classmate",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Senpai",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Kind",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Swim Shorts",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Mechanic",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Stubborn",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Naive",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Non-blood-related Son",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Protective",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bow Tie",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Swimming",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Honest",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Magic Tattoo",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Brave",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Survival",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Idealist",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Psychometrist",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Gakuran",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Eleventh Grader",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Summoning",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Sugiyama Noriaki",
        "original": "杉山 紀彰",
        "lang": "ja"
      }
    ],
    "age": 17,
    "height": 167,
    "weight": 58
  },
  {
    "id": "c33",
    "name": "Saber",
    "original": "セイバー",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/34/34.jpg"
    },
    "description": "Saber is [url=/c15]Shirou's[/url] Servant. Loyal, independent, and reserved, Saber acts coldly but is actually suppressing her emotions to focus on her goals.\n\nHer class is considered the \"Most Outstanding\" with excellent ratings in all categories. Since her Master cannot adequately provide her with Mana, she minimizes her activity to preserve her energy. Saber is frustrated by Shirou's \"protective\" tendencies, believing his erratic and reckless behavior will jeopardize her chances of winning the Holy Grail War.\n\nHer real name is [spoiler]Artoria Pendragon/Altria Pendragon, the legendary King Arthur.[/spoiler]\n\n[From [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Hair Tie",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Green",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Brave",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Ahoge",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Competitive",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Rain Coat",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Dress",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Pleated Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Puffy Sleeves",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "String Ribbon Tie",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ankle Boots",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Off-The-Shoulder Shirt",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Sidehair",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Gauntlet",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Greave",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jeans",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Pantyhose",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Plate Armor",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Longsword",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Long",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Kuudere",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Tassets",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Breastplate",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Stoic",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Invisibility",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Braided Bun",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Reserved",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Named Weapon",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Shoulder-length",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sabaton",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superstrength",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Small Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superhero",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Bodyguard",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Knight",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Spirit",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Strict",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Foreigner",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Honorable",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sandals",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Blond",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Teasing",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Tank Top",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bikini",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Shinai",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Proactive",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Loyal",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Swordsmanship",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Corset",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Tsurime",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Blouse",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Kawasumi Ayako",
        "original": "川澄 綾子",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 154,
    "weight": 42
  },
  {
    "id": "c34",
    "name": "Toosaka Rin",
    "original": "遠坂 凛",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/37/37.jpg"
    },
    "description": "Rin is a model student and idol of [url=/c15]Shirou's[/url] school. She barely talks to other students in her school and exhibits a desire to be left alone, as exemplified by her tendency to stay on the school's rooftop, away from the rest of the students.\n\nShe is also secretly a magus and a Master in the Holy Grail War. Rin is reared as the successor to her family's magecraft, instructed by her father, Toosaka Tokiomi, to prioritize sorcery over her own interests. After her father is killed, Rin continues to perfect her sorcery, with some guidance from [url=/c42]Kotomine Kirei[/url].\n\nShe strives to earn the admiration of her peers to avoid drawing attention to her private life, even though she is secretly stingy, tomboyish, and a slacker. She is sharp, resourceful, and avidly competitive.\n\n[From [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]\n\n[spoiler]She also appears in Fate/Extra CCC as a hidden, optional boss. This is not to be confused with the [url=http://vndb.org/c3529]Toosaka Rin[/url] in the same game.[/spoiler]",
    "traits": [
      {
        "group_name": "Clothes",
        "name": "Cross Design",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Mischievous",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Hair Tie",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted in Middle",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Blue",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Magic Tattoo",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Competitive",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "School \"Go Home Club\" Member",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Pajamas",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Pleated Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Living Alone",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jacket",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Vest",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Miniskirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Loafers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Thigh-high Stockings",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Twin Tails",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Modern Tsundere",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Long",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Pretending",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Reserved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Curtained",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Temporary Protagonist",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Planning",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Eleventh Grader",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Friend",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Medium Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Ward",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Magic/ESP Combat",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Black",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Teasing",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Arrogant",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Classmate",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Senpai",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Kind",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bikini",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "School Uniform",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sweater",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Genius",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Proactive",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Swimming",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Magician",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Serious",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Orphan",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Cooking",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Confident",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Old-fashioned",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Mature",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Tsurime",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Blouse",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Honor Student",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Towel",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Turtleneck Sweater",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Tie",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Ueda Kana",
        "original": "植田 佳奈",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 159,
    "weight": 47
  },
  {
    "id": "c35",
    "name": "Matou Sakura",
    "original": "間桐 桜",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/36/36.jpg"
    },
    "description": "A student in her first year of high school and the sister of [url=/c41]Matou Shinji[/url]. After [url=/c15]Shirou's[/url] father Kiritsugu died, Sakura often visited Shirou's home to help him with his daily chores.\n\nThough [url=/c41]Shinji[/url] is from a family of sorcerers, he asserts that she does not know her family's craft or history.\n\nSakura is outwardly timid but possesses high inner strength. She has a long-standing crush on Shirou.\n\n[From [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Hair Accessory",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Violet",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Apron",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Keikogi",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Sensitive",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Cardigan",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Vest",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bracer",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Hakama",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Loafers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Knee-high Socks",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Bow",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Long",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Deredere",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Shy",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Low Self-esteem",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Kyujutsu",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "School Kyudo Club Member",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Granddaughter",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Tenth Grader",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Friend",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Big Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Asymmetrical Sidehair",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Cleaning",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Schoolmate",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Kouhai",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Timid",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Kind",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Court Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bikini",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "School Uniform",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Smart",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Hard Worker",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Swimming",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Orphan",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Cooking",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Violet",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Tareme",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Pendant Necklace",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Blouse",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Younger Sister",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "V Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Turtleneck Sweater",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Tie",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Shitaya Noriko",
        "original": "下屋 則子",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 156,
    "weight": 46
  },
  {
    "id": "c36",
    "name": "Archer",
    "original": "アーチャー",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/38/38.jpg"
    },
    "description": "A mysterious warrior of unspecified origins and era who serves as the Servant to [url=/c34]Toosaka Rin[/url]. Exuding a palpable aura of cynicism and sarcasm, Archer's worldview starkly contrasts with that of [url=/c15]Shirou's[/url], whose idealistic and naive aspiration to 'save everyone' he perceives as an unattainable folly.\n\n[spoiler]Archer is Emiya Shirou from a possible future, a magus specializing in psychometry and object replication with a focus on swords. Though his replicas may be weaker, he can enhance them to surpass even the originals. In addition, true to his talent since his youth, his unparalleled marksmanship complements his magical prowess, making him a formidable and versatile combatant.[/spoiler]\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]\n\n[spoiler]He also appears as an optional boss in Fate/Extra CCC. This is not to be confused with the [url=http://vndb.org/c3525]playable Archer[/url] in that same game.[/spoiler]",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Mysterious",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "No Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "White",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Grey",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Young-adult",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Muscular",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Dual-Wield Weapon",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Pragmatic",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Sharp-tongued",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trench Coat",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Tanned",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Blunt",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Slicked Back",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ankle Boots",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Plate Armor",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Bow",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Widow's Peak",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Ore",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Invisibility",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cynic",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Named Weapon",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Turtleneck Tank Top",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superstrength",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Tall",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Bodyguard",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Cleaning",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Sarcasm",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Spirit",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Archery",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superhuman",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trousers",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Dao",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Hard Worker",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Rapier",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Swordsmanship",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Suwabe Jun'ichi",
        "original": "諏訪部 順一",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 187,
    "weight": 78
  },
  {
    "id": "c37",
    "name": "Illyasviel von Einzbern",
    "original": "イリヤスフィール・フォン・アインツベルン",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/39/39.jpg"
    },
    "description": "A Teutonic aristocrat, she traveled to Japan to participate in the Holy Grail War as a Master. Illyasviel is a young girl with an unprecedented degree of magical power. Illyasviel asks those she feels comfortable around to simply call her \"Ilya\".\n\n[spoiler]She lives in a castle on the outskirts of Fuyuki City, accompanied only by her [url=/c40]Servant[/url] and two maids.[/spoiler]\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Mischievous",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted to Side",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Optimist",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "White",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Red",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Kid",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Third Person",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Magic Tattoo",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Pragmatic",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trench Coat",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Dress",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Pajamas",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Pleated Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ushanka",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cruel",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Scarf",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Shirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "T-shirt",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Sidehair",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Loafers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sport Bloomers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Boots",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Thigh-high Boots",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Knee-high Socks",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Mayadere",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Long",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Airhead",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Spoiled",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cheerful",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Curtained",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Friendly",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Small Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Imouto Act",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Noble",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Teasing",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Arrogant",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Wealthy",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bikini",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Smart",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Master",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Magician",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Tareme",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Cravat",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Kadowaki Mai",
        "original": "門脇 舞以",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 133,
    "weight": 34
  },
  {
    "id": "c38",
    "name": "Lancer",
    "original": "ランサー",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/40/40.jpg"
    },
    "description": "Lancer is a fierce but fair warrior who enjoys combating a worthy opponent. He has a playful attitude and takes a carefree approach to life but quickly works himself into a frenzy during a heated battle.\n\nLancers are hailed as the \"Most Agile\" of Servant classes. Lancer's weapon, the Gáe Bolg, is a cursed spear that can inflict wounds that do not heal as long as the spear remains in the world.\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Clothes",
        "name": "Pendant Earrings",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Latex",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Red",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Young-adult",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Muscular",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Catsuit",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Spaulders",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Ponytail",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Plate Armor",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Spear",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Widow's Peak",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Long",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Ore",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Selfish",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Carefree",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Named Weapon",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sabaton",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superstrength",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superspeed",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superhero",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Knight",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Based on a Fictional Character",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Sarcasm",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Spirit",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Honorable",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Blue",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Flirting",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Warrior",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Kanna Nobutoshi",
        "original": "神奈 延年",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 185,
    "weight": 70
  },
  {
    "id": "c39",
    "name": "Rider",
    "original": "ライダー",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/41/41.jpg"
    },
    "description": "[spoiler]The servant of [url=/c41]Matou Shinji[/url].[/spoiler]\n\nRider is silent, sultry, and vigilant, never hesitating to shield her Master from harm.\n\nWith her Master incapable of lending Mana to her, she is forced to seek alternative means to augment her abilities. To compensate for her weakness, Rider employs covert battle tactics and takes full advantage of her territory.\n\nHer weapon of choice is a pair of long iron nails fastened to opposite ends of a single chain, which she effectively thrusts from distances.\n\n[spoiler]Rider's most notable ability is her mystic eyes of petrification, which can petrify ones with weak magic resistance and hamper others. Using her magical eyes, Rider can fight much more efficiently. However, she almost always keeps them covered to hide her identity and to avoid harming her allies. [/spoiler]\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "No Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted in Middle",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hidden",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Young-adult",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Ankle Length",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Tattoo",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Taciturn",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Mask",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bracer",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Sidehair",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Thigh-high Boots",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Chain Weapon",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Curtained",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Strapless Mini-dress",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superstrength",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Big Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superspeed",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Bodyguard",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Based on a Fictional Character",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Spirit",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Riding",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Collar",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Blindfold",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Detached Sleeves",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Loyal",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Serious",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Violet",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Asakawa Yuu",
        "original": "浅川 悠",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 172,
    "weight": 57
  },
  {
    "id": "c40",
    "name": "Berserker",
    "original": "バーサーカー",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/42/42.jpg"
    },
    "description": "A formidable warrior, standing tall with skin almost impenetrable by anything less than the finest weapons. He wields a massive ax sword, capable of wreaking havoc with each powerful swing.\n\nAs a Berserker, he has lost his sanity and reason, behaving wildly in combat. Berserkers are esteemed as members of the \"Strongest\" class, but their lack of rationality makes them notoriously difficult to control and prone to turning on their Masters. Despite this, Ilya has complete control over Berserker, keeping him in check.\n\n[spoiler]Berserker's Noble Phantasm, \"Godhand,\" grants him the ability to self-resurrect up to eleven times. This means that it will take twelve kills to defeat Berserker permanently. [/spoiler]\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Greek",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Amber",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "No Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Red",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Adult",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Muscular",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Giant",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Barefoot",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Shaggy",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Unnatural Skin Tone",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Unnatural Sclera",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Sanpaku Eyes",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bracer",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Greave",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Half-Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Plate Armor",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Half-naked",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Axe",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Widow's Peak",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Long",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Tassets",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Wavy",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Insanity",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Demigod",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Primitive",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superstrength",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Bodyguard",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Based on a Fictional Character",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Black",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Spirit",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Belt",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superhuman",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Protective",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Warrior",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Loyal",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Swordsmanship",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Saizen Tadahisa",
        "original": "西前 忠久",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 253,
    "weight": 311
  },
  {
    "id": "c41",
    "name": "Matou Shinji",
    "original": "間桐 慎二",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/43/43.jpg"
    },
    "description": "[url=/c35]Matou Sakura's[/url] older brother, and a long-time friend of [url=/c15]Shirou[/url]. Shinji is very popular as vice-captain of the archery club despite being chauvinistic and a narcissist.\n\n[spoiler]Like Toosaka Rin, he is of a distinguished lineage of sorcerers, though the Matou blood has thinned and no longer produces heirs naturally capable of sorcery.[/spoiler]\n\nShinji feels uncomfortable with his sister's daily visits to Shirou's home. He has an open crush on [url=/c34]Rin[/url], but she does not return his favors.\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Sister Complex",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted in Middle",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Blue",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Coward",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jacket",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jeans",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Boku",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Wavy",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Narcissist",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Grandiose",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Curtained",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Neuroticism",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Full Brother",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "School Kyudo Club Member",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Grandson",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Gakuran",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Eleventh Grader",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Friend",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Student Club Vice President",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Arrogant",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Classmate",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Belt",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Short-tempered",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fishing",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trousers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Dress Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Blue",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Popular",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Older Brother",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Sly",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Kamiya Hiroshi",
        "original": "神谷 浩史",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 167,
    "weight": 57
  },
  {
    "id": "c42",
    "name": "Kotomine Kirei",
    "original": "言峰 綺礼",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/44/44.jpg"
    },
    "description": "Through an agreement between the Mages' Association and the Church, the Holy Grail Wars are moderated by a priest. Kirei's father was the mediator for the fourth Holy Grail War, and when he died, Kirei felt obligated to replace him. His selection as a mediator is unusual, for Kirei is a member of both the Church and Mages' Association, organizations on hostile terms with one another.\n\nSince the death of Rin's father, he has acted as an indirect caretaker for [url=/c34]Rin[/url]. [spoiler]Ranked far lower in the Church hierarchy than Ciel (from Tsukihime), Kirei employs many of the same techniques and weapons she does.[/spoiler] He is surprisingly convincing as a magus, having trained under Rin's father.\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Exorcist",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted in Middle",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Adult",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Muscular",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Talkative",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trench Coat",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Blunt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Cross Necklace",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "T-shirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Collarino",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cynic",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Curtained",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Mullet",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Permanent Blank Eyes",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superstrength",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Legal Guardian",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Priest",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Tall",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Teacher",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Sarcasm",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cold-hearted",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superhuman",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Brown",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Brown",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trousers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Dress Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Magician",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Cross",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Nakata Jouji",
        "original": "中田 譲治",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 193,
    "weight": 82
  },
  {
    "id": "c43",
    "name": "Fujimura Taiga",
    "original": "藤村 大河",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/45/45.jpg"
    },
    "description": "Fujimura Taiga is an English teacher at Shirou's school, homeroom instructor of [url=/c15]Shirou's[/url] class and the supervising teacher for the archery club. She is widely called \"Tiger\", a nickname she dislikes (\"Taiga\" and \"Tiger\" sound similar in Japanese), though Shirou refers to her as \"Fuji-nee\" (older sister Fuji).\n\nAfter Kiritsugu died, Taiga became Shirou's guardian and has been living with him for several years. Taiga and Shirou are very close, and she regards him as a younger brother.\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Hotblooded",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Optimist",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Student Club Advisor",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Young-adult",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Keikogi",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Earrings",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Butt Monkey",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Hakama",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Airhead",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Tiger Cosplay",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Homeroom Teacher",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Friendly",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sleeveless Dress",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Medium Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Legal Guardian",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Cleaning",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Kind",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Court Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Brown",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Kendo",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Brown",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Funny",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sweater",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Food Lover",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Shinai",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Cooking",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Immature",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Tareme",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Itou Miki",
        "original": "伊藤 美紀",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 165,
    "weight": 53
  },
  {
    "id": "c44",
    "name": "Kuzuki Souichirou",
    "original": "葛木 宗一郎",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/46/46.jpg"
    },
    "description": "Kuzuki Souichirou is the strict homeroom instructor of [url=/c34]Rin's[/url] class. He primarily lectures on World History, though he also holds classes in Ethics. A well-built man with angular glasses, Kuzuki is well respected among staff and students, but he is widely regarded as austere and impersonal.\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Eyes",
        "name": "Grey",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Adult",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Suit",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Glasses",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Blunt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jacket",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Necktie",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Stoic",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Homeroom Teacher",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Permanent Blank Eyes",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Student Council Advisor",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Tall",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Formal Shirt",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Teacher",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Black",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Strict",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cold-hearted",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trousers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Dress Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Hard Worker",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "V Bangs",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Nakata Kazuhiro",
        "original": "中多 和宏",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 180,
    "weight": 70
  },
  {
    "id": "c45",
    "name": "Caster",
    "original": "キャスター",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/47/47.jpg"
    },
    "description": "An honored guest of the Ryuudou family permitted to live at their temple until the preparations for her marriage have been finalized. A gorgeous and talented yet mysterious woman of high stature, her presence has attracted the attention of many trainee monks.\n\n[spoiler]She is a Caster-class Servant, who is an expert in using ancient thaumaturgy. \nHer real name is Medea[/spoiler]\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted in Middle",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hidden",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Young-adult",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Jealous",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Dress",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Robe",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Puffy Sleeves",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Hood",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Choker",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Cloak",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Sidehair",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Gloves",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Long",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Deredere",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Tiny Braid",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Ara Ara",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Grandiose",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Curtained",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Planning",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Medium Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Magic/ESP Combat",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Based on a Fictional Character",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Spirit",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Court Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Possessive",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Blue",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Magician",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Loyal",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Tanaka Atsuko",
        "original": "田中 敦子",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 163,
    "weight": 51
  },
  {
    "id": "c46",
    "name": "The Shadow",
    "original": "黒い影",
    "gender": [],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/49/49.jpg"
    },
    "description": "A mysterious being that prowls the streets of Fuyuki City at night, killing countless people. No one knows who or what it is and where it comes from.\n\n[spoiler]After the end of the previous Grail War, her grandfather implanted a piece of the Grail into Sakura in an attempt to recreate it. This led to a remnant of the Third Holy Grail War, the eighth Servant Avenger, attempting to summon himself by controlling Sakura with an alternate persona, The Shadow.[/spoiler]\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Dark",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Non-human",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "No Name",
        "spoiler": 0
      }
    ],
    "va": [],
    "age": null,
    "height": null,
    "weight": null
  },
  {
    "id": "c47",
    "name": "Gilgamesh",
    "original": "ギルガメッシュ",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/50/50.jpg"
    },
    "description": "[spoiler]An arrogant, selfish, and enigmatic Archer-class Servant. He appears familiar with Fuyuki City and has an unrequited love for [url=/c33]Saber[/url].\n\nTypically clad in ornate golden armor, he possesses countless prototypes of other heroes' Noble Phantasms inside his Gate of Babylon. Still, only two (besides Gate of Babylon itself) are truly representative of his true identity.[/spoiler]\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Clothes",
        "name": "Pendant Earrings",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "No Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Optimist",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Red",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Young-adult",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Insightful",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Sharp-tongued",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Blunt",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cruel",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Spaulders",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jacket",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Misanthrope",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Slicked Back",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Shirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Gauntlet",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Greave",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Half-Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Charismatic",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Fur Coat",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Plate Armor",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Widow's Peak",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Tassets",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Ore",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Breastplate",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Invisibility",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Spoiled",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Grandiose",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Incorruptible",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sabaton",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superstrength",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Summoning",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Tall",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Based on a Fictional Character",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Spirit",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Foreigner",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Arrogant",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cold-hearted",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Blond",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superhuman",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Stubborn",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trousers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Dress Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ring",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Smart",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Proactive",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Curse",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Swordsmanship",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Villain",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Slit Pupils",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Seki Tomokazu",
        "original": "関 智一",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 182,
    "weight": 68
  },
  {
    "id": "c48",
    "name": "Ryuudou Issei",
    "original": "柳洞 一成",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/51/51.jpg"
    },
    "description": "Ryuudou Issei is the Student Body President of Homurabara Highschool, and a close friend of [url=/c15]Emiya Shirou[/url]. Issei often asks Shirou to fix broken equipment for the Student Body to try and reduce save money on the school's bill. Issei and Shirou often have lunch together in the Student Body's office where they talk about what ever comes to mind.\n\nHis father is the head priest of Ryuudouji Temple, and his brother was a classmate of [url=/c43]Taiga[/url] and Neko-san. He dislikes [url=/c34]Toosaka Rin[/url] and seems to have some sort of fear of females, although he seems to be unusually at ease around [url=/c33]Saber[/url].\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Blue",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Curtained",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sweater",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trousers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Dress Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Hard Worker",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Blue",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Glasses",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jacket",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jeans",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Ore",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Reserved",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Full Brother",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Friend",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Student Council President",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Strict",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Classmate",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Younger Brother",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted to Side",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Shogi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Gakuran",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Eleventh Grader",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Madono Mitsuaki",
        "original": "真殿 光昭",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 170,
    "weight": 58
  },
  {
    "id": "c49",
    "name": "Assassin",
    "original": "アサシン",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/52/52.jpg"
    },
    "description": "Clothed in a traditional Japanese umanori hakama and kimono with an indigo haori, this handsome Servant [spoiler]bars entry beyond the main gate of the Ryuudouji Temple.[/spoiler]\n\nQuiet and composed, he maintains a degree of bushido and is eager to spar with other Servants. [spoiler]Although he has the same class, he is not to be confused with another Servant known as True Assassin.[/spoiler]\n\nHis real name is [spoiler]Sasaki Kojirou[/spoiler].\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Hair Tie",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "No Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted in Middle",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Blue",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Young-adult",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Kimono",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bracer",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Zouri",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Sidehair",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Hakama",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Ponytail",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Tabi",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "No-dachi",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Waist Length+",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Curtained",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Named Weapon",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Archaic Dialect",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jinbaori",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Samurai",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Based on a Real Person",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Spirit",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Honorable",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Relaxed",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Blue",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Hard Worker",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Swordsmanship",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Miki Shin'ichirou",
        "original": "三木 眞一郎",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 176,
    "weight": 63
  },
  {
    "id": "c50",
    "name": "True Assassin",
    "original": "真アサシン",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/53/53.jpg"
    },
    "description": "True Assassin is an exceptionally dangerous Servant who only makes an appearance in Heaven's Feel. The designation of \"True\" is meant to distinguish this Assassin from [url=/c49]another Servant[/url] that has usurped his place in the Fifth Grail War.\n\nAll True Assassins share a collective identity and have appeared in every Holy Grail War throughout history. They are infamous for an ebony and tattered costume, as well as for concealing their identity behind an ashen skeletal mask.\n\n[Edited from [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Bald",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hidden",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Old",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Muscular",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Unnatural Skin Tone",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Mask",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Cloak",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Half-naked",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bandages",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Throwing Knife",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Assassin",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Based on a Real Person",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Spirit",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cautious",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Loyal",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Rags",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Killing in Battle",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Tall",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superspeed",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Inada Tetsu",
        "original": "稲田 徹",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 215,
    "weight": 62
  },
  {
    "id": "c51",
    "name": "Matou Zouken",
    "original": "間桐 臓硯",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/54/54.jpg"
    },
    "description": "Zouken is a shrewd, powerful, but ancient sorcerer who is patriarch of the Matou family and is known to be the grandfather of Shinji and Sakura.\n\nHe has amassed a large and extensive library of sorcerous knowledge, yet his lessons are strict and harsh. Zouken considers [url=/c41]Shinji[/url] to be a disgrace of a sorcerer, and believes [url=/c35]Sakura[/url] may have more potential than her brother. \n\n[From [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Bald",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "White",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Old",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Kimono",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Hakama",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Walking Stick",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Grandiose",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Grandfather",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Strict",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "No Eyebrows",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Washi",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Wrinkles",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Unnatural Sclera",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Ambitious",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cruel",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Shadowed",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Sanpaku Eyes",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Zouri",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Underweight",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Tabi",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Short",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Tsukayama Masane",
        "original": "津嘉山 正種",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 145,
    "weight": 43
  },
  {
    "id": "c52",
    "name": "Mitsuzuri Ayako",
    "original": "美綴 綾子",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/55/55.jpg"
    },
    "description": "A stellar athlete, and captain of the archery club. Ayako and [url=/c34]Rin[/url] are friends and actively competing to see who will be the first to snag a boyfriend. Ayako is also friends with Shirou and has a one-sided rivalry with him.\n\n[From [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Brown",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Bob Cut",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Curtained",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Shoulder-length",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Brown",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "School Uniform",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Hard Worker",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Keikogi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Vest",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Hakama",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jeans",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Knee-high Socks",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Friendly",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Full Sister",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Friend",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Classmate",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Senpai",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Hoodie",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Blouse",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Older Sister",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Tie",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Atashi",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted in Middle",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Loafers",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Kyujutsu",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "School Kyudo Club President",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Eleventh Grader",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Medium Breasts",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Mizusawa Fumie",
        "original": "水沢 史絵",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 162,
    "weight": 50
  },
  {
    "id": "c53",
    "name": "Dark Sakura",
    "original": "黒 桜",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/56/56.jpg"
    },
    "description": "[spoiler]Once Sakura rejects [url=/c51]Zouken's[/url] control and regains her central consciousness, she becomes Dark Sakura, a rancorous and psychotic distortion of her normal self. In this form, her reasoning and emotions are severely impaired and she is indiscriminately destructive to everything around her.[/Spoiler]\n\n[Edited from [url=http://typemoon.wikia.com/wiki/Sakura]Typemoon Wikia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "White",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Long",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Magician",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Red",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Tattoo",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Yangire",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Grandiose",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Neuroticism",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Full Sister",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Alter Ego",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Non-blood-related Sister",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Psychosis",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Vindictive",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Younger Sister",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "V Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Turtleneck Dress",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Hair Accessory",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Barefoot",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cruel",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Permanent Blank Eyes",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Big Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Asymmetrical Sidehair",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Shitaya Noriko",
        "original": "下屋 則子",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 156,
    "weight": 46
  },
  {
    "id": "c54",
    "name": "Saber Alter",
    "original": "セイバー・オルタナティブ",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/57/57.jpg"
    },
    "description": "When Saber is consumed by [url=/c46]The Shadow[/url], she turns into [url=/c35]Sakura's[/url] Servant, Saber Alter, a corrupted version of her former self with unmatched strength.\n\n[Edited from [url=http://typemoon.wikia.com/wiki/Saber_Alter]Typemoon Wikia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Blond",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Bun",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Odango",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Swordsmanship",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Villain",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Amber",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Tattoo",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Dress",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Mask",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Gauntlet",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Boots",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Plate Armor",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Longsword",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Stoic",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Monarch",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Bodyguard",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Knight",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Based on a Fictional Character",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Strict",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Honorable",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Cold-hearted",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Curse",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Pessimist",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Named Weapon",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superstrength",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Superspeed",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Evil Form",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Kawasumi Ayako",
        "original": "川澄 綾子",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 154,
    "weight": 42
  },
  {
    "id": "c55",
    "name": "Dark Berserker",
    "original": null,
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/58/58.jpg"
    },
    "description": "[spoiler]After Berserker was devoured by the Shadow, it regurgitated him as Dark Berserker, a decrepit and blind version of his former self.[/spoiler]\n\n[From [url=http://en.wikipedia.org/wiki/List_of_Fate/stay_night_characters]Wikipedia[/url]]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Black",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Long",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Wavy",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Warrior",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Fighting",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Red",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Adult",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Muscular",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Giant",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Unnatural Skin Tone",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Tattoo",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Bracer",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Half-Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Plate Armor",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Half-naked",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Axe",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Insanity",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Based on a Fictional Character",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Spirit",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Blindness",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Saizen Tadahisa",
        "original": "西前 忠久",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 253,
    "weight": 311
  },
  {
    "id": "c41594",
    "name": "Hotaruzuka Otoko",
    "original": "蛍塚 音子",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/67/49967.jpg"
    },
    "description": "Otoko works at and is the daughter of the owner of Copenhagen. Otoko went to school with and is friends with both Taiga and Reikan.",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Atashi",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted in Middle",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Young-adult",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Apron",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Geta",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Waist Length+",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Curtained",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Drinking",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Smoking",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Medium Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Braid",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Coworker",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Brown",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Funny",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trousers",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Proactive",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Immature",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Itome",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Blouse",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Fang",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Shop Clerk",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Noda Junko",
        "original": "野田 順子",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 171,
    "weight": 56
  },
  {
    "id": "c41602",
    "name": "Sella",
    "original": "セラ",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/75/49975.jpg"
    },
    "description": "Sella is one of Illyasviel's maids and also acts as her teacher. Sella is strict and difficult to please.",
    "traits": [
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Red",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Young-adult",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Hat",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Boots",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Bullying",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Maid",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Homunculus",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Strict",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Maid's Dress",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Teasing",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Stubborn",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Protective",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Antisocial",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Vindictive",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Loyal",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Old-fashioned",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Mischievous",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Puffy Sleeves",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Terada Haruhi",
        "original": "寺田 はるひ",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 163,
    "weight": 49
  },
  {
    "id": "c41603",
    "name": "Leysritt",
    "original": "リーゼリット",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/76/49976.jpg"
    },
    "description": "Leysritt is a combat-specialized homunculus that is one of Illyasviel's maids. Leysritt is blunt, but has trouble communicating.",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "White",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Red",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Young-adult",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Puffy Sleeves",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Hat",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Blunt",
        "spoiler": 0
      },
      {
        "group_name": "Items",
        "name": "Halberd",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Boots",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Stoic",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Friendly",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Permanent Blank Eyes",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Big Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Maid",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Bodyguard",
        "spoiler": 0
      },
      {
        "group_name": "Engages in",
        "name": "Teasing",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Homunculus",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Maid's Dress",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Protective",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Loyal",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Tsurime",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Miyagawa Miho",
        "original": "宮川 美保",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 163,
    "weight": 52
  },
  {
    "id": "c41605",
    "name": "Saegusa Yukika",
    "original": "三枝 由紀香",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/78/49978.jpg"
    },
    "description": "Yukika is close friends with Kaede and Kane. Yukika comes from a poor family and has a patient and caring personality.",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Capri Pants",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "School Athletic Club Member",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Vest",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Loafers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Knee-high Socks",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Shy",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Shoulder-length",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Full Sister",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Eleventh Grader",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Friend",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Medium Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Classmate",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Poor",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Kind",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Brown",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Brown",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "School Uniform",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sweater",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Popular",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Tareme",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sports Uniform",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Blouse",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Older Sister",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Tie",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Nakao Eri",
        "original": "中尾 衣里",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 155,
    "weight": 39
  },
  {
    "id": "c41606",
    "name": "Makidera Kaede",
    "original": "蒔寺 楓",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/79/49979.jpg"
    },
    "description": "A friend of Rin and talented sprinter. Makidera is loud, energetic, and rarely seen away from her close friends Yukika and Kane.",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Black",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Parted to Side",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Talkative",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Olive",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Competitive",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "School Athletic Club Member",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Jacket",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Vest",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Loud",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Shorts",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sports Shoes",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Airhead",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Eleventh Grader",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Friend",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Small Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Black",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Classmate",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Short-tempered",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Funny",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "School Uniform",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Energetic",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Tsurime",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sports Uniform",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Blouse",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Fang",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Tie",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Yuimoto Michiru",
        "original": "結下 みちる",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 163,
    "weight": 49
  },
  {
    "id": "c41607",
    "name": "Himuro Kane",
    "original": "氷室 鐘",
    "gender": [
      "f"
    ],
    "image": {
      "dims": [
        208,
        300
      ],
      "url": "https://t.vndb.org/ch/80/49980.jpg"
    },
    "description": "Kane is close friends with Kaede and Saegusa. Kane has a quiet and serious personality.",
    "traits": [
      {
        "group_name": "Engages in",
        "name": "Not Sexually Involved",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hazel",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Curious",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "School Athletic Club Member",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Taciturn",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Watashi",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Glasses",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Blunt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Necktie",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Vest",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Sidehair",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Skirt",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Loafers",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Pantyhose",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Long",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Reserved",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Permanent Blank Eyes",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Straight",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Eleventh Grader",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Friend",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Medium Breasts",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Classmate",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Teen",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "School Uniform",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Strange",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Smart",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Tsurime",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Sports Uniform",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Blouse",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Grey",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Blunt Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Ribbon Tie",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Nakagawa Rie",
        "original": "中川 里江",
        "lang": "ja"
      }
    ],
    "age": null,
    "height": 157,
    "weight": 48
  },
  {
    "id": "c96619",
    "name": "Emiya Kiritsugu",
    "original": "衛宮 切嗣",
    "gender": [
      "m"
    ],
    "image": {
      "dims": [
        256,
        144
      ],
      "url": "https://t.vndb.org/ch/17/116717.jpg"
    },
    "description": "[url=/c15]Shirou's[/url] foster father and his savior from the aftermath of the massive Fuyuki fire.\n[spoiler]Prior to his fateful meeting with Shirou, Kiritsugu was notorious as the \"Magus Killer,\" a freelancer who would hunt down heretical magi. He was also a Participant in the Fourth Holy Grail War[/spoiler]",
    "traits": [
      {
        "group_name": "Hair",
        "name": "Spiky Bangs",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Black",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Adult",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Shaggy",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Pragmatic",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Coat",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Idealist",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Vest",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Boku",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Permanent Blank Eyes",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Short",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Present in Flashbacks Only",
        "spoiler": 0
      },
      {
        "group_name": "Subject of",
        "name": "Pre-Story Death",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Retired",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Loose Necktie",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Formal Shirt",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Black",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Altruistic",
        "spoiler": 0
      },
      {
        "group_name": "Personality",
        "name": "Kind",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Slim",
        "spoiler": 0
      },
      {
        "group_name": "Body",
        "name": "Pale",
        "spoiler": 0
      },
      {
        "group_name": "Clothes",
        "name": "Trousers",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Non-blood-related Father",
        "spoiler": 0
      },
      {
        "group_name": "Role",
        "name": "Magician",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Stubble",
        "spoiler": 0
      },
      {
        "group_name": "Eyes",
        "name": "Hosome",
        "spoiler": 0
      },
      {
        "group_name": "Hair",
        "name": "Spiky",
        "spoiler": 0
      }
    ],
    "va": [
      {
        "name": "Koyama Rikiya",
        "original": "小山 力也",
        "lang": "ja"
      }
    ],
    "age": 34,
    "height": 175,
    "weight": 67
  }
];
