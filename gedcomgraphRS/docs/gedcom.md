# GEDCOM support

The parser reads GEDCOM 5.5.1 line format: level, optional `@id@`, tag, value. Person and family ids keep their file form minus the `@` signs, so `@I3@` becomes `I3`. References to records that do not exist are ignored instead of failing the parse.

## Person records

| Tag    | Use                                              |
| ------ | ------------------------------------------------ |
| `NAME` | Display name. Only the given part renders        |
| `SEX`  | `M`, `F`, `U`, or anything else for card borders |
| `BIRT` | Birth date from its `DATE` child                 |
| `CHR`, `BAPM` | Christening date, used when no birth date exists |
| `DEAT`, `BURI` | Death. Marks the card dead and draws the ribbon |
| `TITL` | Title lines, one per value                       |
| `FAMC` | Parent family link                               |
| `FAMS` | Spouse family link                               |

Any other level-1 tag lands in the generic fact list with its value, date, and place children. The date fallback at the bottom of a card uses the first date found on any fact.

## Family records

| Tag          | Use                                |
| ------------ | ---------------------------------- |
| `HUSB`       | Husband link, in file order        |
| `WIFE`       | Wife link, in file order           |
| `CHIL`       | Child link                         |
| `MARR`       | Marriage date from its `DATE` child, shown on the bond |

Spouse lookup takes the first husband, then the first wife, then any extra partners, capped at two people per family.

## Dates on cards

A birth date renders with a `★` prefix, a christening with `≈`, a death with `✛`. Birth and death share one line when both are short (7 characters or fewer each, counted in characters, not bytes). Longer dates stack on two lines.
