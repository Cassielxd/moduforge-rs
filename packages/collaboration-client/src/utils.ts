import * as Y from 'yjs';

/**
 * Parses a Yjs update by applying it to a temporary document
 * and returning the full resulting state. This is more reliable than
 * trying to diff the changes.
 * @param update The Yjs update Uint8Array.
 * @returns A JSON object representing the document state after the update.
 */
export function parseYjsUpdate(update: Uint8Array): Record<string, any> {
    try {
        const tempDoc = new Y.Doc();
        Y.applyUpdate(tempDoc, update);
        
        const incrementalState = tempDoc.getMap('nodes').toJSON();
        
        return {
            success: true,
            updateSize: update.length,
            incrementalContent: incrementalState,
        };

    } catch (error: any) {
        return {
            success: false,
            error: error.message,
            updateSize: update.length,
        };
    }
}