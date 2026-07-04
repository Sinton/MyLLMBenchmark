import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export async function listenToEvent<T>(
  eventName: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(eventName, (event) => handler(event.payload));
}
