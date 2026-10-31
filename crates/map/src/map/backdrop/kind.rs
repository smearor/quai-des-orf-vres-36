use serde::Deserialize;
use serde::Serialize;

/// The kind of a [`crate::Landmark`], choosing its rendering icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LandmarkKind {
    /// Monument/tower (Tour Eiffel, Fernsehturm, Brandenburger Tor).
    Monument,
    /// Museum/gallery (Louvre, Musée d'Orsay, Museuminsel).
    Museum,
    /// Church/cathedral (Notre-Dame, Gedächtniskirche).
    Church,
    /// Park/green area (Jardin du Luxembourg, Tiergarten).
    Park,
    /// Square/plaza (Potsdamer Platz, Place de la Concorde).
    Square,
    /// Railway terminus (Gare du Nord, Hauptbahnhof).
    Station,
    /// Airport (Orly, BER).
    Airport,
    /// Water body or waterway point marker (Canal Saint-Martin,
    /// Landwehrkanal) - river courses are `water` polylines; this
    /// marks named spots on them.
    Waterway,
    /// Bridge or crossing (Pont Neuf, Oberbaumbrücke, Pont Alexandre III).
    Bridge,
    /// Island in a river/lake (Île de la Cité, Pfaueninsel).
    Island,
    /// Hill, elevated vantage point, or lookout (Montmartre,
    /// Teufelsberg, Kreuzberg).
    Hill,
    /// Theatre, opera house, concert hall, or cabaret (Moulin Rouge,
    /// Philharmonie Berlin, Friedrichstadt-Palast).
    Venue,
    /// Stadium, arena, or sports complex (Stade de France,
    /// Olympiastadion Berlin, Parc des Princes).
    Stadium,
    /// Zoo, aquarium, or botanical garden (Ménagerie du Jardin des
    /// Plantes, Zoologischer Garten Berlin, Tierpark).
    Zoo,
    /// Palace, castle, or royal residence (Château de Versailles,
    /// Palais du Luxembourg, Schloss Bellevue).
    Palace,
    /// Historic city gate, wall, or fortification (Porte Saint-Denis,
    /// Zitadelle Spandau).
    Fortification,
    /// Government building or city hall (Hôtel de Ville, Rotes
    /// Rathaus, Bundeskanzleramt).
    Civic,
    /// Department store, historic market, or shopping arcade
    /// (Galeries Lafayette, KaDeWe, Passages Couverts).
    Market,
    /// Prominent street, boulevard, or avenue (Champs-Élysées, Unter
    /// den Linden, Kurfürstendamm).
    Boulevard,
    /// High-rise, skyscraper, or business district (Tour
    /// Montparnasse, La Défense, Kollhoff-Tower).
    Skyscraper,
}
