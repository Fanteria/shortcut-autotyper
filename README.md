# Shortcut AutoTyper

[![codecov](https://codecov.io/gh/Fanteria/shortcut-autotyper/graph/badge.svg?token=1KI46G2Y0I)](https://codecov.io/gh/Fanteria/shortcut-autotyper)

Shortcut AutoTyper is a command-line tool that generates sequences and combinations from a configuration file and types the generated strings on the keyboard. It provides an easy way to automate typing repetitive sequences using predefined shortcuts.

## Configuration File

Shortcut AutoTyper requires a configuration file in JSON format to define sequences and combinations. The default path for the configuration file is `$HOME/.shortcut_autotyper.json`. The configuration file should have the following structure:

``` json
{
  "combinations": {
    "X": "A2 c B3..6 N",
    "Y": "C3 X2"
  },
  "sequences": {
    "A": "A1",
    "B": "B1_",
    "AB": "can be more than one char",
    "c": "keys are key sensitive",
    "D": "🐧 emotes can be used too",
    "N": "\n",
  }
}
```

Every key in sequences and combinations must be unique and combinations are separated by spaces.

## Usage

Below is the general command format:
`shortcut-autotyper [SEQUENCE/COMBINATION][COUNT/RANGE]`

The program performs error handling to ensure proper usage. If an invalid name or combination is provided, the program will throw an error with a detailed description of the issue. For instance, using spaces in the names or combinations may result in an error in combinations, but in the command line, they will be interpreted as two separated names.

### Examples
Type the sequence named "A" three times:
```
shortcut-autotyper A3
```
Type the combination named "X" four times:
```
shortcut-autotyper X4
```
Type the sequence named "N" three, four, or five times:
```
shortcut-autotyper numbers 3..6
```
Type multiple names:
```
shortcut-autotyper A B2 c3 d4..6
```

## Contributions
Bug reports are highly welcome! If you encounter any issues or have feature suggestions, please don't hesitate to create an issue on the GitHub repository. Your input and feedback are invaluable in helping us improve Shortcut AutoTyper.
