// Function to get all commands from db
import { get_all_keybindings } from './document';


export type ExecuteFunction = () => void;

export type KeyboardInput = {
    keyDown: string,
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
    moveLeft?: () => void;
    moveRight?: () => void;
    moveUp?: () => void;
    moveDown?: () => void;
    switchToDocument1?: () => void;
    switchToDocument2?: () => void;
    switchToDocument3?: () => void;
    switchToDocument4?: () => void;
    switchToDocument5?: () => void;
    switchToDocument6?: () => void;
    switchToDocument7?: () => void;
    switchToDocument8?: () => void;
    switchToDocument9?: () => void;
    [key: string]: (() => void) | undefined;
}

export class keybindings {
    // Static map of active keybindings (default + custom)
    static activeBindings: Record<string, KeyboardInput> = {};
    
    // Default keybindings
    static defaultBindings(): Record<string, KeyboardInput> {
        return {
            bold: {
                keyDown: "b",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            italic: {
                keyDown: "i",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            underline: {
                keyDown: "u",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            openColorPicker: {
                keyDown: "f",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            moveLeft: {
                keyDown: "h",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            moveRight: {
                keyDown: "l",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            moveUp: {
                keyDown: "k",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            moveDown: {
                keyDown: "j",
                altDown: false,
                ctrlDown: false,
                shiftDown: false,
            },
            switchToDocument1: {
                keyDown: "1",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument2: {
                keyDown: "2",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument3: {
                keyDown: "3",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument4: {
                keyDown: "4",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument5: {
                keyDown: "5",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument6: {
                keyDown: "6",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument7: {
                keyDown: "7",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument8: {
                keyDown: "8",
                altDown: false,
                ctrlDown: true,
                shiftDown: false,
            },
            switchToDocument9: {
                keyDown: "9",
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
    };
    
    // Command name to function name mapping
    static commandToFunctionMap: Record<string, string> = {
        'bold': 'applyBoldFormatting',
        'italic': 'applyItalicFormatting',
        'underline': 'applyUnderlineFormatting',
        'openColorPicker': 'openColorPicker',
        'moveLeft': 'moveLeft',
        'moveRight': 'moveRight',
        'moveUp': 'moveUp',
        'moveDown': 'moveDown',
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
        const keyDown = parts[parts.length - 1].trim();
        
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
                keyDown,
                altDown,
                ctrlDown,
                shiftDown
            }
        });
        
        return {
            keyDown,
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
            const key = `${input.keyDown}_${input.altDown}_${input.ctrlDown}_${input.shiftDown}`;
            
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
            keyDown: event.key,
            altDown: event.altKey,
            ctrlDown: event.ctrlKey,
            shiftDown: event.shiftKey
        };
    }
    
    // Helper to convert KeyboardInput to a map key
    static getMapKey(input: KeyboardInput): string {
        return `${input.keyDown}_${input.altDown}_${input.ctrlDown}_${input.shiftDown}`;
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