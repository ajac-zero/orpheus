# Limiting the provider list

You can use the `only` and `ignore` parameters to define the providers that you want to use/avoid.

These parameters accept an iterable of `Provider` objects.

This opens up some interesing posibilities. For example, let's use Kimi K2 only through Groq for lighting-fast inference.

```rust
use orpheus::prelude::*;

fn main() {
    let client = Orpheus::new("Your-API-Key");

    let res = client
        .chat("Tell me about the Romans.")
        .model("moonshotai/kimi-k2")
        .with_preferences(|pref| pref.only([Provider::Groq]))
        .send()
        .unwrap();

    println!("Provider: {}", res.provider);
    println!("Response: {}", res.content().unwrap());
}
```

```
Provider: Groq
Response: The Romans were the people—from shepherds to emperors—who built one of the most influential civilizations in world history. Their story spans roughly twelve centuries, but historians usually divide it into three big chapters: the Monarchy (753–509 BC), the Republic (509–27 BC), and the Empire (27 BC–AD 476 in the West, continuing to 1453 in the East as the Byzantine Empire).

What made the Romans exceptional was not a single invention or conquest, but a talent for organization, adaptation, and sheer persistence. They borrowed Greek drama, Etruscan engineering, Carthaginian naval design, and Gaulish saddles, then blended them with their own rigid discipline and practical mindset. A quick tour:

1. Engineering & Infrastructure
   • Roads—nearly 80,000 km of paved, cambered highways that allowed legions and merchants to move at the same speed in Britain as in Syria.
   • Aqueducts—structures like the Pont du Gard and Segovia’s elevated channels still carry water today.
   • The arch and concrete revolutionized construction; the Pantheon’s dome (AD 128) remains the world’s largest unreinforced concrete dome.

2. Law & Administration
   • “Civil law” (jus civile) evolved into concepts still encoded in modern European legal systems: contracts, property rights, the trust.
   • Provinces governed by a mix of legionary forts, town councils (curiae), and local elites—an early form of federalism.

3. Army & Society
   • The classic Roman legion—5,000 heavy infantry plus auxiliaries—was as much a construction corps as a fighting force; soldiers built the roads they marched on.
   • Roman citizenship was gradually extended, first to Italians (after the Social War 91–88 BC) and eventually to every free adult male in the Empire (Constitutio Antoniniana, AD 212).

4. Culture & Spectacle
   • Literature—Virgil’s Aeneid recasts Rome’s mythic past; Ovid’s Metamorphoses becomes medieval Europe’s window into Greek myth.
   • The Colosseum could be flooded for mock naval battles; chariot races in the Circus Maximus riveted crowds of 150,000.
   • Gladiators, at once reviled and idolized, symbolized the Roman knack for turning brutality into mass entertainment.

5. Religion
   • A polytheistic pantheon mirrored Greek gods but carried a strong streak of contractual obligation (“I give so that you give”).
   • Rome’s greatest miracle, many Romans argued, was the rise and spread of Christianity—starting as a persecuted sect (Nero’s scapegoat for the fire of Rome, AD 64) and ending as the Empire’s official religion under Constantine’s Edict of Milan (AD 313).

6. Decline & Legacy
   • Inflation, civil wars, and external pressures (Goths crossing the Danube in 376) shattered the western half; but the East—capital in Byzantium—endured for another thousand years.
   • Latin never vanished; it evolved into the Romance languages (Spanish, French, Romanian, Portuguese, Italian) and underpins 60 % of English vocabulary.

Fun epilogues:
   • Our calendar (Julian/Gregorian) comes from Julius Caesar’s 46 BC reform.
   • The eagle on the U.S. Great Seal? A deliberate echo of Rome’s aquila standard.
   • The legal term pro bono and the phrase “crossing the Rubicon” still carry Roman weight.

In short, the Romans were the world’s supreme cultural recyclers, able to govern from the deserts of Arabia to the banks of the Rhine with a few thousand miles of stone and parchment.
```
