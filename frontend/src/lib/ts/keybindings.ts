// Function to get all commands from db
import { get_all_keybindings } from './account';


export type ExecuteFunction = () => void;

export type KeyboardInput = {
    kd: string,
    altDown: boolean,
    ctrlDown: boolean,
    shiftDown: boolean,
}

// Interface for returning keybinding data from backend
interface BackendKeybinding {
	user_id: number;
	command_id: number;
	keybinding: string;
}

// Command IDs mapped to their names for lookups
interface CommandIdMap {
    [id: number]: string;
}

// Command functions interface
export interface CommandFunctions {
    applyBoldFormatting?: () => void;
    applyItalicFormatting?: () => void;
    applyUnderlineFormatting?: () => void;
    openColorPicker?: () => void;
    enterInsertMode?: () => void;
    moveLeft?: () => void;
    moveRight?: () => void;
    moveUp?: () => void;
    moveDown?: () => void;
    moveToStartOfLine?: () => void;
    moveToEndOfLine?: () => void;
    switchToDocument1?: () => void;
    switchToDocument2?: () => void;
    switchToDocument3?: () => void;
    switchToDocument4?: () => void;
    switchToDocument5?: () => void;
    switchToDocument6?: () => void;
    switchToDocument7?: () => void;
    switchToDocument8?: () => void;
    switchToDocument9?: () => void;
    moveToEndOfDocument?: () => void;
    moveToStartOfDocument?: () => void;
    toggleCommandSheet?: () => void;
    findNextMatch?: () => void;
    findPreviousMatch?: () => void;
    deleteSelectedText?: () => void;
    yankText?: () => void;
    deleteLine?: () => void;
    pasteText?: () => Promise<void>;
    [key: string]: (() => void | Promise<void>) | undefined;
}

export class keybindings {
    // Static map of active keybindings (default + custom)
    static activeBindings: Record<string, KeyboardInput> = {};
    
    // Default keybindings
    static defaultBindings(): Record<string, KeyboardInput> {
        return {
            bold: {
                kd: "b",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            italic: {
                kd: "i",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            underline: {
                kd: "u",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            enterInsertMode: {
                kd: "i",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            openColorPicker: {
                kd: "f",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            moveLeft: {
                kd: "h",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            moveRight: {
                kd: "l",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            moveUp: {
                kd: "k",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            moveDown: {
                kd: "j",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            moveToStartOfLine: {
                kd: "0",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            moveToEndOfLine: {
                kd: "$",
                altDown: false,
                ctrlDown: false,
                shiftDown: true,
            },
            moveToEndOfDocument: {
                kd: "G",
                altDown: false,
                ctrlDown: false,
                shiftDown: true,
            },
            moveToStartOfDocument: {
                kd: "g",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            toggleCommandSheet: {
                kd: "/",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            findNextMatch: {
                kd: "n",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            findPreviousMatch: {
                kd: "m",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            deleteSelectedText: {
                kd: "x",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            yankText: {
                kd: "y",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            deleteLine: {
                kd: "d",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            pasteText: {
                kd: "p",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            switchToDocument1: {
                kd: "1",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument2: {
                kd: "2",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument3: {
                kd: "3",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument4: {
                kd: "4",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument5: {
                kd: "5",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument6: {
                kd: "6",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument7: {
                kd: "7",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument8: {
                kd: "8",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument9: {
                kd: "9",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
        }
    }
    
    // Command ID to name mapping
    static commandIdToName: CommandIdMap = {
        1: 'bold',
        2: 'italic',
        3: 'underline',
        4: 'openColorPicker',
        5: 'moveLeft',
        6: 'moveRight',
        7: 'moveUp',
        8: 'moveDown',
        9: 'switchToDocument1',
        10: 'switchToDocument2',
        11: 'switchToDocument3',
        12: 'switchToDocument4',
        13: 'switchToDocument5',
        14: 'switchToDocument6',
        15: 'switchToDocument7',
        16: 'switchToDocument8',
        17: 'switchToDocument9',
        18: 'enterInsertMode',
        19: 'moveToStartOfLine',
        20: 'moveToEndOfLine',
        21: 'moveToEndOfDocument',
        22: 'moveToStartOfDocument',
        23: 'toggleCommandSheet',
        24: 'findNextMatch',
        25: 'findPreviousMatch',
        26: 'deleteSelectedText',
        27: 'yankText',
        28: 'deleteLine',
        29: 'pasteText',
    };
    
    // Command name to function name mapping
    static commandToFunctionMap: Record<string, string> = {
        'bold': 'applyBoldFormatting',
        'italic': 'applyItalicFormatting',
        'underline': 'applyUnderlineFormatting',
        'openColorPicker': 'openColorPicker',
        'enterInsertMode': 'enterInsertMode',
        'moveLeft': 'moveLeft',
        'moveRight': 'moveRight',
        'moveUp': 'moveUp',
        'moveDown': 'moveDown',
        'moveToStartOfLine': 'moveToStartOfLine',
        'moveToEndOfLine': 'moveToEndOfLine',
        'moveToEndOfDocument': 'moveToEndOfDocument',
        'moveToStartOfDocument': 'moveToStartOfDocument',
        'toggleCommandSheet': 'toggleCommandSheet',
        'findNextMatch': 'findNextMatch',
        'findPreviousMatch': 'findPreviousMatch',
        'deleteSelectedText': 'deleteSelectedText',
        'yankText': 'yankText',
        'deleteLine': 'deleteCurrentLine',
        'pasteText': 'pasteText',
        'switchToDocument1': 'switchToDocument1',
        'switchToDocument2': 'switchToDocument2',
        'switchToDocument3': 'switchToDocument3',
        'switchToDocument4': 'switchToDocument4',
        'switchToDocument5': 'switchToDocument5',
        'switchToDocument6': 'switchToDocument6',
        'switchToDocument7': 'switchToDocument7',
        'switchToDocument8': 'switchToDocument8',
        'switchToDocument9': 'switchToDocument9',
    };
    
    // Helper function to parse keybinding string from backend
    static parseKeybindingString(keybinding: string): KeyboardInput {
        // First split by spaces and join with + to normalize the format
        const normalizedKeybinding = keybinding.toLowerCase().split(' ').join('+');
        
        // Now split by + to separate modifiers from key
        const parts = normalizedKeybinding.split('+');
        
        // The last part is always the key
        const kd = parts[parts.length - 1].trim();
        
        let ctrlDown = false;
        let altDown = false;
        let shiftDown = false;
        
        // Check for modifiers in all parts except the last one
        parts.slice(0, -1).forEach(part => {
            const mod = part.trim().toLowerCase();
            if (mod === 'ctrl' || mod === 'control') ctrlDown = true;
            if (mod === 'alt') altDown = true;
            if (mod === 'shift') shiftDown = true;
        });
        
        console.debug('Parsed keybinding:', {
            original: keybinding,
            normalized: normalizedKeybinding,
            parts,
            result: {
                kd,
                altDown,
                ctrlDown,
                shiftDown
            }
        });
        
        return {
            kd,
            altDown,
            ctrlDown,
            shiftDown
        };
    }
    
    // Function to initialize keybindings with default values
    static initializeBindings(): void {
        // Set the default bindings as active
        this.activeBindings = { ...this.defaultBindings() };
    }
    
    // API function to fetch custom keybindings and update the active bindings
    static async fetchAndUpdateBindings(): Promise<void> {
        try {
            // Initialize with defaults first
            this.initializeBindings();
            
            // Fetch custom keybindings from backend
            const customBindings = await get_all_keybindings();
            
            if (!customBindings) {
                console.error("Failed to fetch custom keybindings");
                return;
            }
            
            // Update active bindings with custom ones
            customBindings.forEach((binding: BackendKeybinding) => {
                const commandName = this.commandIdToName[binding.command_id];
                
                if (commandName && binding.keybinding) {
                    // Parse the keybinding string into KeyboardInput format
                    const parsedBinding = this.parseKeybindingString(binding.keybinding);
                    
                    // Update the active binding for this command
                    this.activeBindings[commandName] = parsedBinding;
                }
            });
            
            console.log("Updated keybindings:", this.activeBindings);
	} catch (error) {
            console.error("Error fetching and updating keybindings:", error);
        }
    }
    
    // Function to get a specific keybinding
    static getBinding(commandName: string): KeyboardInput | undefined {
        return this.activeBindings[commandName];
    }
    
    // Get command name by ID
    static getCommandNameById(commandId: number): string | undefined {
        return this.commandIdToName[commandId];
    }
    
    // Get function name by command name
    static getFunctionNameByCommand(commandName: string): string | undefined {
        return this.commandToFunctionMap[commandName];
    }
    
    // Get function name directly by command ID
    static getFunctionNameById(commandId: number): string | undefined {
        const commandName = this.getCommandNameById(commandId);
        if (commandName) {
            return this.getFunctionNameByCommand(commandName);
        }
        return undefined;
    }
}

// Map of keyboard input object to execution
export class keybindingMap {
    static keybindingMapper(commandFunctions: CommandFunctions): Record<string, ExecuteFunction> {
        const map: Record<string, ExecuteFunction> = {};
        
        // Generate keys for each active binding
        Object.entries(keybindings.activeBindings).forEach(([commandName, input]) => {
            const key = `${input.kd}_${input.altDown}_${input.ctrlDown}_${input.shiftDown}`;
            
            // Get the function name associated with this command
            const functionName = keybindings.getFunctionNameByCommand(commandName);
            
            if (functionName && commandFunctions[functionName]) {
                // Map this key combination to the actual function from page.svelte
                map[key] = () => {
                    console.log(`Executing command: ${commandName} via ${functionName}`);
                    commandFunctions[functionName]?.();
                };
            } else {
                console.warn(`No function found for command: ${commandName}`);
            }
        });
        
        return map;
    }
    
    // Helper to convert a KeyboardEvent to our input format
    static keyEventToInput(event: KeyboardEvent): KeyboardInput {
        return {
            // Convert key to lowercase for consistent matching
            kd: event.key.toLowerCase(), 
            altDown: event.altKey,
            ctrlDown: event.ctrlKey,
            shiftDown: event.shiftKey
        };
    }
    
    // Helper to convert KeyboardInput to a map key
    static getMapKey(input: KeyboardInput): string {
        // Ensure this uses the same lowercase key property (kd)
        return `${input.kd}_${input.altDown}_${input.ctrlDown}_${input.shiftDown}`;
    }
    
    // Handle keyboard input
    static handleKeyboardInput(event: KeyboardEvent, commandFunctions: CommandFunctions): boolean {
        const input = this.keyEventToInput(event);
        const mapKey = this.getMapKey(input);
        const map = this.keybindingMapper(commandFunctions);
        
        if (map[mapKey]) {
            event.preventDefault(); // Prevent default browser behavior
            map[mapKey]();
            return true;
	}
	
	return false;
    }
}