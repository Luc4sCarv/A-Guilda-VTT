export type PersonalAsset = { id:string; name:string; kind:'art'|'audio'|'cover'; file:Blob };

function database():Promise<IDBDatabase>{
  return new Promise((resolve,reject)=>{
    const request=indexedDB.open('tormenta-vtt-assets',1);
    request.onupgradeneeded=()=>request.result.createObjectStore('assets',{keyPath:'id'});
    request.onsuccess=()=>resolve(request.result);
    request.onerror=()=>reject(request.error);
  });
}

async function operation<T>(mode:IDBTransactionMode,act:(store:IDBObjectStore)=>IDBRequest<T>):Promise<T>{
  const db=await database();
  return new Promise((resolve,reject)=>{
    const transaction=db.transaction('assets',mode);
    const request=act(transaction.objectStore('assets'));
    let result:T;
    request.onsuccess=()=>{result=request.result;};
    request.onerror=()=>reject(request.error);
    transaction.oncomplete=()=>{db.close();resolve(result);};
    transaction.onerror=()=>{db.close();reject(transaction.error);};
  });
}

export function listPersonalAssets():Promise<PersonalAsset[]>{return operation('readonly',store=>store.getAll());}
export function getPersonalAsset(id:string):Promise<PersonalAsset|undefined>{return operation('readonly',store=>store.get(id));}
export function putPersonalAsset(item:PersonalAsset):Promise<IDBValidKey>{return operation('readwrite',store=>store.put(item));}
export async function addPersonalAsset(file:File,kind:'art'|'audio'):Promise<PersonalAsset>{
  if(!file.size||file.size>(kind==='art'?3_000_000:50_000_000))throw new Error(kind==='art'?'Arte deve ter até 3 MB':'Música deve ter até 50 MB');
  if(!(kind==='art'?/\.(png|jpe?g|webp)$/i:/\.(mp3|ogg|wav)$/i).test(file.name))throw new Error('Formato de arquivo não aceito');
  const item={id:crypto.randomUUID(),name:file.name.replace(/\.[^.]+$/,'').slice(0,80),kind,file};
  await operation('readwrite',store=>store.add(item));
  return item;
}
