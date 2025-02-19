crate::region! {
    course: FieldLight,
    name: "Kakariko Village",
    color: Name,
    village {
        locations: [
            "Stylish Woman's House Crack": None @Crack(IndoorLight 14[7] StylishWoman),
            "Sahasrahla's House Crack": None @Crack(16[275] SahasrahlasHouse),

            "Kakariko Village Weather Vane": None @WeatherVane(16[203] KakarikoVillageWV),

            "Bee Guy (1)": ItemInsectNet @Event(IndoorLight/FieldLight_18_InsectNet[0xB]),
            "Bee Guy (2)": BadgeBee @Event(IndoorLight/FieldLight_18_InsectNet[0x1F]),
            "Dodge the Cuccos": HeartPiece @Event(FieldLight_29_Kokko[0x67]),
            "Kakariko Item Shop (1)": EscapeFruit @None(),
            "Kakariko Item Shop (2)": StopFruit @None(),
            "Kakariko Item Shop (3)": ItemShield @None(),
            "Kakariko Jail": RupeeSilver @Chest(IndoorLight 3[3]),
            "Kakariko Well (Bottom)": RupeeR @Chest(CaveLight 4[6]),
            "Kakariko Well (Top)": HeartPiece @Heart(CaveLight 4[8]),
            "Rupee Rush (Hyrule)": HeartPiece @Event(FieldLight_28_Minigame[0x26]),
            "Shady Guy": ItemRentalHookShot @Event(FieldLight_18_Touzoku[0x12]),
            "Street Merchant (Left)": ItemBottle @Shop(Merchant(0)),
            "Street Merchant (Right)": ItemStoneBeauty @Shop(Merchant(2)),
            "Stylish Woman": HeartPiece @Event(IndoorLight/FieldLight_18_ClosedHouse[4]),
            "Woman": RupeeR @Event(IndoorLight/FieldLight_18_MiddleLady[0xF]),
            "[Mai] Cucco Ranch Tree": Maiamai @Maiamai(24[42]),
            "[Mai] Hyrule Rupee Rush Wall": Maiamai @Maiamai(45[46]),
            "[Mai] Kakariko Bush": Maiamai @Maiamai(16[304]),
            "[Mai] Kakariko Sand": Maiamai @Maiamai(16[393]),
            "[Mai] Woman's Roof": Maiamai @Maiamai(16[394]),
        ],
    },
}
