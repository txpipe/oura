// deno-fmt-ignore-file
// deno-lint-ignore-file
// This code was bundled using `deno bundle` and it's not recommended to edit it manually

function getEnv(name) {
    try {
        return Deno.env.get(name);
    } catch (err) {
        if (err instanceof Deno.errors.PermissionDenied) {
            return undefined;
        }
        throw err;
    }
}
const encoder = new TextEncoder();
const decoder = new TextDecoder();
function concat(...buffers) {
    const size = buffers.reduce((acc, { length  })=>acc + length, 0);
    const buf = new Uint8Array(size);
    let i = 0;
    buffers.forEach((buffer)=>{
        buf.set(buffer, i);
        i += buffer.length;
    });
    return buf;
}
const encodeBase64 = (input)=>{
    let unencoded = input;
    if (typeof unencoded === 'string') {
        unencoded = encoder.encode(unencoded);
    }
    const CHUNK_SIZE = 0x8000;
    const arr = [];
    for(let i = 0; i < unencoded.length; i += CHUNK_SIZE){
        arr.push(String.fromCharCode.apply(null, unencoded.subarray(i, i + 0x8000)));
    }
    return btoa(arr.join(''));
};
const encode = (input)=>{
    return encodeBase64(input).replace(/=/g, '').replace(/\+/g, '-').replace(/\//g, '_');
};
const decodeBase64 = (encoded)=>{
    return new Uint8Array(atob(encoded).split('').map((c)=>c.charCodeAt(0)));
};
const decode = (input)=>{
    let encoded = input;
    if (encoded instanceof Uint8Array) {
        encoded = decoder.decode(encoded);
    }
    encoded = encoded.replace(/-/g, '+').replace(/_/g, '/').replace(/\s/g, '');
    try {
        return decodeBase64(encoded);
    } catch  {
        throw new TypeError('The input to be decoded is not correctly encoded.');
    }
};
class JOSEError extends Error {
    static get code() {
        return 'ERR_JOSE_GENERIC';
    }
    code = 'ERR_JOSE_GENERIC';
    constructor(message){
        super(message);
        this.name = this.constructor.name;
        Error.captureStackTrace?.(this, this.constructor);
    }
}
class JOSENotSupported extends JOSEError {
    static get code() {
        return 'ERR_JOSE_NOT_SUPPORTED';
    }
    code = 'ERR_JOSE_NOT_SUPPORTED';
}
class JWSInvalid extends JOSEError {
    static get code() {
        return 'ERR_JWS_INVALID';
    }
    code = 'ERR_JWS_INVALID';
}
class JWTInvalid extends JOSEError {
    static get code() {
        return 'ERR_JWT_INVALID';
    }
    code = 'ERR_JWT_INVALID';
}
class JWKSInvalid extends JOSEError {
    static get code() {
        return 'ERR_JWKS_INVALID';
    }
    code = 'ERR_JWKS_INVALID';
}
class JWKSNoMatchingKey extends JOSEError {
    static get code() {
        return 'ERR_JWKS_NO_MATCHING_KEY';
    }
    code = 'ERR_JWKS_NO_MATCHING_KEY';
    message = 'no applicable key found in the JSON Web Key Set';
}
class JWKSMultipleMatchingKeys extends JOSEError {
    static get code() {
        return 'ERR_JWKS_MULTIPLE_MATCHING_KEYS';
    }
    code = 'ERR_JWKS_MULTIPLE_MATCHING_KEYS';
    message = 'multiple matching keys found in the JSON Web Key Set';
}
function isCryptoKey(key) {
    try {
        return key != null && typeof key.extractable === 'boolean' && typeof key.algorithm.name === 'string' && typeof key.type === 'string';
    } catch  {
        return false;
    }
}
crypto.getRandomValues.bind(crypto);
function isCloudflareWorkers() {
    return typeof WebSocketPair === 'function';
}
function isNodeJs() {
    try {
        return process.versions.node !== undefined;
    } catch  {
        return false;
    }
}
function unusable(name, prop = 'algorithm.name') {
    return new TypeError(`CryptoKey does not support this operation, its ${prop} must be ${name}`);
}
function isAlgorithm(algorithm, name) {
    return algorithm.name === name;
}
function getHashLength(hash) {
    return parseInt(hash.name.slice(4), 10);
}
function getNamedCurve(alg) {
    switch(alg){
        case 'ES256':
            return 'P-256';
        case 'ES384':
            return 'P-384';
        case 'ES512':
            return 'P-521';
        default:
            throw new Error('unreachable');
    }
}
function checkUsage(key, usages) {
    if (usages.length && !usages.some((expected)=>key.usages.includes(expected))) {
        let msg = 'CryptoKey does not support this operation, its usages must include ';
        if (usages.length > 2) {
            const last = usages.pop();
            msg += `one of ${usages.join(', ')}, or ${last}.`;
        } else if (usages.length === 2) {
            msg += `one of ${usages[0]} or ${usages[1]}.`;
        } else {
            msg += `${usages[0]}.`;
        }
        throw new TypeError(msg);
    }
}
function checkSigCryptoKey(key, alg, ...usages) {
    switch(alg){
        case 'HS256':
        case 'HS384':
        case 'HS512':
            {
                if (!isAlgorithm(key.algorithm, 'HMAC')) throw unusable('HMAC');
                const expected = parseInt(alg.slice(2), 10);
                const actual = getHashLength(key.algorithm.hash);
                if (actual !== expected) throw unusable(`SHA-${expected}`, 'algorithm.hash');
                break;
            }
        case 'RS256':
        case 'RS384':
        case 'RS512':
            {
                if (!isAlgorithm(key.algorithm, 'RSASSA-PKCS1-v1_5')) throw unusable('RSASSA-PKCS1-v1_5');
                const expected = parseInt(alg.slice(2), 10);
                const actual = getHashLength(key.algorithm.hash);
                if (actual !== expected) throw unusable(`SHA-${expected}`, 'algorithm.hash');
                break;
            }
        case 'PS256':
        case 'PS384':
        case 'PS512':
            {
                if (!isAlgorithm(key.algorithm, 'RSA-PSS')) throw unusable('RSA-PSS');
                const expected = parseInt(alg.slice(2), 10);
                const actual = getHashLength(key.algorithm.hash);
                if (actual !== expected) throw unusable(`SHA-${expected}`, 'algorithm.hash');
                break;
            }
        case isNodeJs() && 'EdDSA':
            {
                if (key.algorithm.name !== 'NODE-ED25519' && key.algorithm.name !== 'NODE-ED448') throw unusable('NODE-ED25519 or NODE-ED448');
                break;
            }
        case isCloudflareWorkers() && 'EdDSA':
            {
                if (!isAlgorithm(key.algorithm, 'NODE-ED25519')) throw unusable('NODE-ED25519');
                break;
            }
        case 'ES256':
        case 'ES384':
        case 'ES512':
            {
                if (!isAlgorithm(key.algorithm, 'ECDSA')) throw unusable('ECDSA');
                const expected = getNamedCurve(alg);
                const actual = key.algorithm.namedCurve;
                if (actual !== expected) throw unusable(expected, 'algorithm.namedCurve');
                break;
            }
        default:
            throw new TypeError('CryptoKey does not support this operation');
    }
    checkUsage(key, usages);
}
const __default = (actual, ...types)=>{
    let msg = 'Key must be ';
    if (types.length > 2) {
        const last = types.pop();
        msg += `one of type ${types.join(', ')}, or ${last}.`;
    } else if (types.length === 2) {
        msg += `one of type ${types[0]} or ${types[1]}.`;
    } else {
        msg += `of type ${types[0]}.`;
    }
    if (actual == null) {
        msg += ` Received ${actual}`;
    } else if (typeof actual === 'function' && actual.name) {
        msg += ` Received function ${actual.name}`;
    } else if (typeof actual === 'object' && actual != null) {
        if (actual.constructor && actual.constructor.name) {
            msg += ` Received an instance of ${actual.constructor.name}`;
        }
    }
    return msg;
};
const types = [
    'CryptoKey'
];
const __default1 = (key)=>{
    return isCryptoKey(key);
};
const isDisjoint = (...headers)=>{
    const sources = headers.filter(Boolean);
    if (sources.length === 0 || sources.length === 1) {
        return true;
    }
    let acc;
    for (const header of sources){
        const parameters = Object.keys(header);
        if (!acc || acc.size === 0) {
            acc = new Set(parameters);
            continue;
        }
        for (const parameter of parameters){
            if (acc.has(parameter)) {
                return false;
            }
            acc.add(parameter);
        }
    }
    return true;
};
function isObjectLike(value) {
    return typeof value === 'object' && value !== null;
}
function isObject(input) {
    if (!isObjectLike(input) || Object.prototype.toString.call(input) !== '[object Object]') {
        return false;
    }
    if (Object.getPrototypeOf(input) === null) {
        return true;
    }
    let proto = input;
    while(Object.getPrototypeOf(proto) !== null){
        proto = Object.getPrototypeOf(proto);
    }
    return Object.getPrototypeOf(input) === proto;
}
const __default2 = (alg, key)=>{
    if (alg.startsWith('RS') || alg.startsWith('PS')) {
        const { modulusLength  } = key.algorithm;
        if (typeof modulusLength !== 'number' || modulusLength < 2048) {
            throw new TypeError(`${alg} requires key modulusLength to be 2048 bits or larger`);
        }
    }
};
const findOid = (keyData, oid, from = 0)=>{
    if (from === 0) {
        oid.unshift(oid.length);
        oid.unshift(0x06);
    }
    let i = keyData.indexOf(oid[0], from);
    if (i === -1) return false;
    const sub = keyData.subarray(i, i + oid.length);
    if (sub.length !== oid.length) return false;
    return sub.every((value, index)=>value === oid[index]) || findOid(keyData, oid, i + 1);
};
const getNamedCurve1 = (keyData)=>{
    switch(true){
        case findOid(keyData, [
            0x2a,
            0x86,
            0x48,
            0xce,
            0x3d,
            0x03,
            0x01,
            0x07
        ]):
            return 'P-256';
        case findOid(keyData, [
            0x2b,
            0x81,
            0x04,
            0x00,
            0x22
        ]):
            return 'P-384';
        case findOid(keyData, [
            0x2b,
            0x81,
            0x04,
            0x00,
            0x23
        ]):
            return 'P-521';
        case (isCloudflareWorkers() || isNodeJs()) && findOid(keyData, [
            0x2b,
            0x65,
            0x70
        ]):
            return 'Ed25519';
        case isNodeJs() && findOid(keyData, [
            0x2b,
            0x65,
            0x71
        ]):
            return 'Ed448';
        default:
            throw new JOSENotSupported('Invalid or unsupported EC Key Curve or OKP Key Sub Type');
    }
};
const genericImport = async (replace, keyFormat, pem, alg, options)=>{
    let algorithm;
    let keyUsages;
    const keyData = new Uint8Array(atob(pem.replace(replace, '')).split('').map((c)=>c.charCodeAt(0)));
    const isPublic = keyFormat === 'spki';
    switch(alg){
        case 'PS256':
        case 'PS384':
        case 'PS512':
            algorithm = {
                name: 'RSA-PSS',
                hash: `SHA-${alg.slice(-3)}`
            };
            keyUsages = isPublic ? [
                'verify'
            ] : [
                'sign'
            ];
            break;
        case 'RS256':
        case 'RS384':
        case 'RS512':
            algorithm = {
                name: 'RSASSA-PKCS1-v1_5',
                hash: `SHA-${alg.slice(-3)}`
            };
            keyUsages = isPublic ? [
                'verify'
            ] : [
                'sign'
            ];
            break;
        case 'RSA-OAEP':
        case 'RSA-OAEP-256':
        case 'RSA-OAEP-384':
        case 'RSA-OAEP-512':
            algorithm = {
                name: 'RSA-OAEP',
                hash: `SHA-${parseInt(alg.slice(-3), 10) || 1}`
            };
            keyUsages = isPublic ? [
                'encrypt',
                'wrapKey'
            ] : [
                'decrypt',
                'unwrapKey'
            ];
            break;
        case 'ES256':
            algorithm = {
                name: 'ECDSA',
                namedCurve: 'P-256'
            };
            keyUsages = isPublic ? [
                'verify'
            ] : [
                'sign'
            ];
            break;
        case 'ES384':
            algorithm = {
                name: 'ECDSA',
                namedCurve: 'P-384'
            };
            keyUsages = isPublic ? [
                'verify'
            ] : [
                'sign'
            ];
            break;
        case 'ES512':
            algorithm = {
                name: 'ECDSA',
                namedCurve: 'P-521'
            };
            keyUsages = isPublic ? [
                'verify'
            ] : [
                'sign'
            ];
            break;
        case 'ECDH-ES':
        case 'ECDH-ES+A128KW':
        case 'ECDH-ES+A192KW':
        case 'ECDH-ES+A256KW':
            algorithm = {
                name: 'ECDH',
                namedCurve: getNamedCurve1(keyData)
            };
            keyUsages = isPublic ? [] : [
                'deriveBits'
            ];
            break;
        case (isCloudflareWorkers() || isNodeJs()) && 'EdDSA':
            const namedCurve = getNamedCurve1(keyData).toUpperCase();
            algorithm = {
                name: `NODE-${namedCurve}`,
                namedCurve: `NODE-${namedCurve}`
            };
            keyUsages = isPublic ? [
                'verify'
            ] : [
                'sign'
            ];
            break;
        default:
            throw new JOSENotSupported('Invalid or unsupported "alg" (Algorithm) value');
    }
    return crypto.subtle.importKey(keyFormat, keyData, algorithm, options?.extractable ?? false, keyUsages);
};
const fromPKCS8 = (pem, alg, options)=>{
    return genericImport(/(?:-----(?:BEGIN|END) PRIVATE KEY-----|\s)/g, 'pkcs8', pem, alg, options);
};
function subtleMapping(jwk) {
    let algorithm;
    let keyUsages;
    switch(jwk.kty){
        case 'oct':
            {
                switch(jwk.alg){
                    case 'HS256':
                    case 'HS384':
                    case 'HS512':
                        algorithm = {
                            name: 'HMAC',
                            hash: `SHA-${jwk.alg.slice(-3)}`
                        };
                        keyUsages = [
                            'sign',
                            'verify'
                        ];
                        break;
                    case 'A128CBC-HS256':
                    case 'A192CBC-HS384':
                    case 'A256CBC-HS512':
                        throw new JOSENotSupported(`${jwk.alg} keys cannot be imported as CryptoKey instances`);
                    case 'A128GCM':
                    case 'A192GCM':
                    case 'A256GCM':
                    case 'A128GCMKW':
                    case 'A192GCMKW':
                    case 'A256GCMKW':
                        algorithm = {
                            name: 'AES-GCM'
                        };
                        keyUsages = [
                            'encrypt',
                            'decrypt'
                        ];
                        break;
                    case 'A128KW':
                    case 'A192KW':
                    case 'A256KW':
                        algorithm = {
                            name: 'AES-KW'
                        };
                        keyUsages = [
                            'wrapKey',
                            'unwrapKey'
                        ];
                        break;
                    case 'PBES2-HS256+A128KW':
                    case 'PBES2-HS384+A192KW':
                    case 'PBES2-HS512+A256KW':
                        algorithm = {
                            name: 'PBKDF2'
                        };
                        keyUsages = [
                            'deriveBits'
                        ];
                        break;
                    default:
                        throw new JOSENotSupported('Invalid or unsupported JWK "alg" (Algorithm) Parameter value');
                }
                break;
            }
        case 'RSA':
            {
                switch(jwk.alg){
                    case 'PS256':
                    case 'PS384':
                    case 'PS512':
                        algorithm = {
                            name: 'RSA-PSS',
                            hash: `SHA-${jwk.alg.slice(-3)}`
                        };
                        keyUsages = jwk.d ? [
                            'sign'
                        ] : [
                            'verify'
                        ];
                        break;
                    case 'RS256':
                    case 'RS384':
                    case 'RS512':
                        algorithm = {
                            name: 'RSASSA-PKCS1-v1_5',
                            hash: `SHA-${jwk.alg.slice(-3)}`
                        };
                        keyUsages = jwk.d ? [
                            'sign'
                        ] : [
                            'verify'
                        ];
                        break;
                    case 'RSA-OAEP':
                    case 'RSA-OAEP-256':
                    case 'RSA-OAEP-384':
                    case 'RSA-OAEP-512':
                        algorithm = {
                            name: 'RSA-OAEP',
                            hash: `SHA-${parseInt(jwk.alg.slice(-3), 10) || 1}`
                        };
                        keyUsages = jwk.d ? [
                            'decrypt',
                            'unwrapKey'
                        ] : [
                            'encrypt',
                            'wrapKey'
                        ];
                        break;
                    default:
                        throw new JOSENotSupported('Invalid or unsupported JWK "alg" (Algorithm) Parameter value');
                }
                break;
            }
        case 'EC':
            {
                switch(jwk.alg){
                    case 'ES256':
                        algorithm = {
                            name: 'ECDSA',
                            namedCurve: 'P-256'
                        };
                        keyUsages = jwk.d ? [
                            'sign'
                        ] : [
                            'verify'
                        ];
                        break;
                    case 'ES384':
                        algorithm = {
                            name: 'ECDSA',
                            namedCurve: 'P-384'
                        };
                        keyUsages = jwk.d ? [
                            'sign'
                        ] : [
                            'verify'
                        ];
                        break;
                    case 'ES512':
                        algorithm = {
                            name: 'ECDSA',
                            namedCurve: 'P-521'
                        };
                        keyUsages = jwk.d ? [
                            'sign'
                        ] : [
                            'verify'
                        ];
                        break;
                    case 'ECDH-ES':
                    case 'ECDH-ES+A128KW':
                    case 'ECDH-ES+A192KW':
                    case 'ECDH-ES+A256KW':
                        algorithm = {
                            name: 'ECDH',
                            namedCurve: jwk.crv
                        };
                        keyUsages = jwk.d ? [
                            'deriveBits'
                        ] : [];
                        break;
                    default:
                        throw new JOSENotSupported('Invalid or unsupported JWK "alg" (Algorithm) Parameter value');
                }
                break;
            }
        case (isCloudflareWorkers() || isNodeJs()) && 'OKP':
            if (jwk.alg !== 'EdDSA') {
                throw new JOSENotSupported('Invalid or unsupported JWK "alg" (Algorithm) Parameter value');
            }
            switch(jwk.crv){
                case 'Ed25519':
                    algorithm = {
                        name: 'NODE-ED25519',
                        namedCurve: 'NODE-ED25519'
                    };
                    keyUsages = jwk.d ? [
                        'sign'
                    ] : [
                        'verify'
                    ];
                    break;
                case isNodeJs() && 'Ed448':
                    algorithm = {
                        name: 'NODE-ED448',
                        namedCurve: 'NODE-ED448'
                    };
                    keyUsages = jwk.d ? [
                        'sign'
                    ] : [
                        'verify'
                    ];
                    break;
                default:
                    throw new JOSENotSupported('Invalid or unsupported JWK "crv" (Subtype of Key Pair) Parameter value');
            }
            break;
        default:
            throw new JOSENotSupported('Invalid or unsupported JWK "kty" (Key Type) Parameter value');
    }
    return {
        algorithm,
        keyUsages
    };
}
const parse = async (jwk)=>{
    const { algorithm , keyUsages  } = subtleMapping(jwk);
    const rest = [
        algorithm,
        jwk.ext ?? false,
        jwk.key_ops ?? keyUsages
    ];
    if (algorithm.name === 'PBKDF2') {
        return crypto.subtle.importKey('raw', decode(jwk.k), ...rest);
    }
    const keyData = {
        ...jwk
    };
    delete keyData.alg;
    return crypto.subtle.importKey('jwk', keyData, ...rest);
};
async function importPKCS8(pkcs8, alg, options) {
    if (typeof pkcs8 !== 'string' || pkcs8.indexOf('-----BEGIN PRIVATE KEY-----') !== 0) {
        throw new TypeError('"pkcs8" must be PCKS8 formatted string');
    }
    return fromPKCS8(pkcs8, alg, options);
}
async function importJWK(jwk, alg, octAsKeyObject) {
    if (!isObject(jwk)) {
        throw new TypeError('JWK must be an object');
    }
    alg ||= jwk.alg;
    if (typeof alg !== 'string' || !alg) {
        throw new TypeError('"alg" argument is required when "jwk.alg" is not present');
    }
    switch(jwk.kty){
        case 'oct':
            if (typeof jwk.k !== 'string' || !jwk.k) {
                throw new TypeError('missing "k" (Key Value) Parameter value');
            }
            octAsKeyObject ??= jwk.ext !== true;
            if (octAsKeyObject) {
                return parse({
                    ...jwk,
                    alg,
                    ext: false
                });
            }
            return decode(jwk.k);
        case 'RSA':
            if (jwk.oth !== undefined) {
                throw new JOSENotSupported('RSA JWK "oth" (Other Primes Info) Parameter value is not supported');
            }
        case 'EC':
        case 'OKP':
            return parse({
                ...jwk,
                alg
            });
        default:
            throw new JOSENotSupported('Unsupported "kty" (Key Type) Parameter value');
    }
}
const symmetricTypeCheck = (key)=>{
    if (key instanceof Uint8Array) return;
    if (!__default1(key)) {
        throw new TypeError(__default(key, ...types, 'Uint8Array'));
    }
    if (key.type !== 'secret') {
        throw new TypeError(`${types.join(' or ')} instances for symmetric algorithms must be of type "secret"`);
    }
};
const asymmetricTypeCheck = (key, usage)=>{
    if (!__default1(key)) {
        throw new TypeError(__default(key, ...types));
    }
    if (key.type === 'secret') {
        throw new TypeError(`${types.join(' or ')} instances for asymmetric algorithms must not be of type "secret"`);
    }
    if (usage === 'sign' && key.type === 'public') {
        throw new TypeError(`${types.join(' or ')} instances for asymmetric algorithm signing must be of type "private"`);
    }
    if (usage === 'decrypt' && key.type === 'public') {
        throw new TypeError(`${types.join(' or ')} instances for asymmetric algorithm decryption must be of type "private"`);
    }
    if (key.algorithm && usage === 'verify' && key.type === 'private') {
        throw new TypeError(`${types.join(' or ')} instances for asymmetric algorithm verifying must be of type "public"`);
    }
    if (key.algorithm && usage === 'encrypt' && key.type === 'private') {
        throw new TypeError(`${types.join(' or ')} instances for asymmetric algorithm encryption must be of type "public"`);
    }
};
const checkKeyType = (alg, key, usage)=>{
    const symmetric = alg.startsWith('HS') || alg === 'dir' || alg.startsWith('PBES2') || /^A\d{3}(?:GCM)?KW$/.test(alg);
    if (symmetric) {
        symmetricTypeCheck(key);
    } else {
        asymmetricTypeCheck(key, usage);
    }
};
function validateCrit(Err, recognizedDefault, recognizedOption, protectedHeader, joseHeader) {
    if (joseHeader.crit !== undefined && protectedHeader.crit === undefined) {
        throw new Err('"crit" (Critical) Header Parameter MUST be integrity protected');
    }
    if (!protectedHeader || protectedHeader.crit === undefined) {
        return new Set();
    }
    if (!Array.isArray(protectedHeader.crit) || protectedHeader.crit.length === 0 || protectedHeader.crit.some((input)=>typeof input !== 'string' || input.length === 0)) {
        throw new Err('"crit" (Critical) Header Parameter MUST be an array of non-empty strings when present');
    }
    let recognized;
    if (recognizedOption !== undefined) {
        recognized = new Map([
            ...Object.entries(recognizedOption),
            ...recognizedDefault.entries()
        ]);
    } else {
        recognized = recognizedDefault;
    }
    for (const parameter of protectedHeader.crit){
        if (!recognized.has(parameter)) {
            throw new JOSENotSupported(`Extension Header Parameter "${parameter}" is not recognized`);
        }
        if (joseHeader[parameter] === undefined) {
            throw new Err(`Extension Header Parameter "${parameter}" is missing`);
        } else if (recognized.get(parameter) && protectedHeader[parameter] === undefined) {
            throw new Err(`Extension Header Parameter "${parameter}" MUST be integrity protected`);
        }
    }
    return new Set(protectedHeader.crit);
}
Symbol();
function subtleDsa(alg, algorithm) {
    const hash = `SHA-${alg.slice(-3)}`;
    switch(alg){
        case 'HS256':
        case 'HS384':
        case 'HS512':
            return {
                hash,
                name: 'HMAC'
            };
        case 'PS256':
        case 'PS384':
        case 'PS512':
            return {
                hash,
                name: 'RSA-PSS',
                saltLength: alg.slice(-3) >> 3
            };
        case 'RS256':
        case 'RS384':
        case 'RS512':
            return {
                hash,
                name: 'RSASSA-PKCS1-v1_5'
            };
        case 'ES256':
        case 'ES384':
        case 'ES512':
            return {
                hash,
                name: 'ECDSA',
                namedCurve: algorithm.namedCurve
            };
        case (isCloudflareWorkers() || isNodeJs()) && 'EdDSA':
            const { namedCurve  } = algorithm;
            return {
                name: namedCurve,
                namedCurve
            };
        default:
            throw new JOSENotSupported(`alg ${alg} is not supported either by JOSE or your javascript runtime`);
    }
}
function getCryptoKey(alg, key, usage) {
    if (isCryptoKey(key)) {
        checkSigCryptoKey(key, alg, usage);
        return key;
    }
    if (key instanceof Uint8Array) {
        if (!alg.startsWith('HS')) {
            throw new TypeError(__default(key, ...types));
        }
        return crypto.subtle.importKey('raw', key, {
            hash: `SHA-${alg.slice(-3)}`,
            name: 'HMAC'
        }, false, [
            usage
        ]);
    }
    throw new TypeError(__default(key, ...types, 'Uint8Array'));
}
const __default3 = (date)=>Math.floor(date.getTime() / 1000);
const hour = 60 * 60;
const day = hour * 24;
const week = day * 7;
const year = day * 365.25;
const REGEX = /^(\d+|\d+\.\d+) ?(seconds?|secs?|s|minutes?|mins?|m|hours?|hrs?|h|days?|d|weeks?|w|years?|yrs?|y)$/i;
const __default4 = (str)=>{
    const matched = REGEX.exec(str);
    if (!matched) {
        throw new TypeError('Invalid time period format');
    }
    const value = parseFloat(matched[1]);
    const unit = matched[2].toLowerCase();
    switch(unit){
        case 'sec':
        case 'secs':
        case 'second':
        case 'seconds':
        case 's':
            return Math.round(value);
        case 'minute':
        case 'minutes':
        case 'min':
        case 'mins':
        case 'm':
            return Math.round(value * 60);
        case 'hour':
        case 'hours':
        case 'hr':
        case 'hrs':
        case 'h':
            return Math.round(value * hour);
        case 'day':
        case 'days':
        case 'd':
            return Math.round(value * day);
        case 'week':
        case 'weeks':
        case 'w':
            return Math.round(value * week);
        default:
            return Math.round(value * year);
    }
};
const sign = async (alg, key, data)=>{
    const cryptoKey = await getCryptoKey(alg, key, 'sign');
    __default2(alg, cryptoKey);
    const signature = await crypto.subtle.sign(subtleDsa(alg, cryptoKey.algorithm), cryptoKey, data);
    return new Uint8Array(signature);
};
class FlattenedSign {
    _payload;
    _protectedHeader;
    _unprotectedHeader;
    constructor(payload){
        if (!(payload instanceof Uint8Array)) {
            throw new TypeError('payload must be an instance of Uint8Array');
        }
        this._payload = payload;
    }
    setProtectedHeader(protectedHeader) {
        if (this._protectedHeader) {
            throw new TypeError('setProtectedHeader can only be called once');
        }
        this._protectedHeader = protectedHeader;
        return this;
    }
    setUnprotectedHeader(unprotectedHeader) {
        if (this._unprotectedHeader) {
            throw new TypeError('setUnprotectedHeader can only be called once');
        }
        this._unprotectedHeader = unprotectedHeader;
        return this;
    }
    async sign(key, options) {
        if (!this._protectedHeader && !this._unprotectedHeader) {
            throw new JWSInvalid('either setProtectedHeader or setUnprotectedHeader must be called before #sign()');
        }
        if (!isDisjoint(this._protectedHeader, this._unprotectedHeader)) {
            throw new JWSInvalid('JWS Protected and JWS Unprotected Header Parameter names must be disjoint');
        }
        const joseHeader = {
            ...this._protectedHeader,
            ...this._unprotectedHeader
        };
        const extensions = validateCrit(JWSInvalid, new Map([
            [
                'b64',
                true
            ]
        ]), options?.crit, this._protectedHeader, joseHeader);
        let b64 = true;
        if (extensions.has('b64')) {
            b64 = this._protectedHeader.b64;
            if (typeof b64 !== 'boolean') {
                throw new JWSInvalid('The "b64" (base64url-encode payload) Header Parameter must be a boolean');
            }
        }
        const { alg  } = joseHeader;
        if (typeof alg !== 'string' || !alg) {
            throw new JWSInvalid('JWS "alg" (Algorithm) Header Parameter missing or invalid');
        }
        checkKeyType(alg, key, 'sign');
        let payload = this._payload;
        if (b64) {
            payload = encoder.encode(encode(payload));
        }
        let protectedHeader;
        if (this._protectedHeader) {
            protectedHeader = encoder.encode(encode(JSON.stringify(this._protectedHeader)));
        } else {
            protectedHeader = encoder.encode('');
        }
        const data = concat(protectedHeader, encoder.encode('.'), payload);
        const signature = await sign(alg, key, data);
        const jws = {
            signature: encode(signature),
            payload: ''
        };
        if (b64) {
            jws.payload = decoder.decode(payload);
        }
        if (this._unprotectedHeader) {
            jws.header = this._unprotectedHeader;
        }
        if (this._protectedHeader) {
            jws.protected = decoder.decode(protectedHeader);
        }
        return jws;
    }
}
class CompactSign {
    _flattened;
    constructor(payload){
        this._flattened = new FlattenedSign(payload);
    }
    setProtectedHeader(protectedHeader) {
        this._flattened.setProtectedHeader(protectedHeader);
        return this;
    }
    async sign(key, options) {
        const jws = await this._flattened.sign(key, options);
        if (jws.payload === undefined) {
            throw new TypeError('use the flattened module for creating JWS with b64: false');
        }
        return `${jws.protected}.${jws.payload}.${jws.signature}`;
    }
}
class ProduceJWT {
    _payload;
    constructor(payload){
        if (!isObject(payload)) {
            throw new TypeError('JWT Claims Set MUST be an object');
        }
        this._payload = payload;
    }
    setIssuer(issuer) {
        this._payload = {
            ...this._payload,
            iss: issuer
        };
        return this;
    }
    setSubject(subject) {
        this._payload = {
            ...this._payload,
            sub: subject
        };
        return this;
    }
    setAudience(audience) {
        this._payload = {
            ...this._payload,
            aud: audience
        };
        return this;
    }
    setJti(jwtId) {
        this._payload = {
            ...this._payload,
            jti: jwtId
        };
        return this;
    }
    setNotBefore(input) {
        if (typeof input === 'number') {
            this._payload = {
                ...this._payload,
                nbf: input
            };
        } else {
            this._payload = {
                ...this._payload,
                nbf: __default3(new Date()) + __default4(input)
            };
        }
        return this;
    }
    setExpirationTime(input) {
        if (typeof input === 'number') {
            this._payload = {
                ...this._payload,
                exp: input
            };
        } else {
            this._payload = {
                ...this._payload,
                exp: __default3(new Date()) + __default4(input)
            };
        }
        return this;
    }
    setIssuedAt(input) {
        if (typeof input === 'undefined') {
            this._payload = {
                ...this._payload,
                iat: __default3(new Date())
            };
        } else {
            this._payload = {
                ...this._payload,
                iat: input
            };
        }
        return this;
    }
}
class SignJWT extends ProduceJWT {
    _protectedHeader;
    setProtectedHeader(protectedHeader) {
        this._protectedHeader = protectedHeader;
        return this;
    }
    async sign(key, options) {
        const sig = new CompactSign(encoder.encode(JSON.stringify(this._payload)));
        sig.setProtectedHeader(this._protectedHeader);
        if (Array.isArray(this._protectedHeader?.crit) && this._protectedHeader.crit.includes('b64') && this._protectedHeader.b64 === false) {
            throw new JWTInvalid('JWTs MUST NOT use unencoded payload');
        }
        return sig.sign(key, options);
    }
}
function getKtyFromAlg(alg) {
    switch(typeof alg === 'string' && alg.slice(0, 2)){
        case 'RS':
        case 'PS':
            return 'RSA';
        case 'ES':
            return 'EC';
        case 'Ed':
            return 'OKP';
        default:
            throw new JOSENotSupported('Unsupported "alg" value for a JSON Web Key Set');
    }
}
function isJWKSLike(jwks) {
    return jwks && typeof jwks === 'object' && Array.isArray(jwks.keys) && jwks.keys.every(isJWKLike);
}
function isJWKLike(key) {
    return isObject(key);
}
function clone(obj) {
    if (typeof structuredClone === 'function') {
        return structuredClone(obj);
    }
    return JSON.parse(JSON.stringify(obj));
}
class LocalJWKSet {
    _jwks;
    _cached = new WeakMap();
    constructor(jwks){
        if (!isJWKSLike(jwks)) {
            throw new JWKSInvalid('JSON Web Key Set malformed');
        }
        this._jwks = clone(jwks);
    }
    async getKey(protectedHeader, token) {
        const joseHeader = {
            ...protectedHeader,
            ...token.header
        };
        const candidates = this._jwks.keys.filter((jwk)=>{
            let candidate = jwk.kty === getKtyFromAlg(joseHeader.alg);
            if (candidate && typeof joseHeader.kid === 'string') {
                candidate = joseHeader.kid === jwk.kid;
            }
            if (candidate && typeof jwk.alg === 'string') {
                candidate = joseHeader.alg === jwk.alg;
            }
            if (candidate && typeof jwk.use === 'string') {
                candidate = jwk.use === 'sig';
            }
            if (candidate && Array.isArray(jwk.key_ops)) {
                candidate = jwk.key_ops.includes('verify');
            }
            if (candidate && joseHeader.alg === 'EdDSA') {
                candidate = jwk.crv === 'Ed25519' || jwk.crv === 'Ed448';
            }
            if (candidate) {
                switch(joseHeader.alg){
                    case 'ES256':
                        candidate = jwk.crv === 'P-256';
                        break;
                    case 'ES256K':
                        candidate = jwk.crv === 'secp256k1';
                        break;
                    case 'ES384':
                        candidate = jwk.crv === 'P-384';
                        break;
                    case 'ES512':
                        candidate = jwk.crv === 'P-521';
                        break;
                    default:
                }
            }
            return candidate;
        });
        const { 0: jwk , length  } = candidates;
        if (length === 0) {
            throw new JWKSNoMatchingKey();
        } else if (length !== 1) {
            throw new JWKSMultipleMatchingKeys();
        }
        const cached = this._cached.get(jwk) || this._cached.set(jwk, {}).get(jwk);
        if (cached[joseHeader.alg] === undefined) {
            const keyObject = await importJWK({
                ...jwk,
                ext: true
            }, joseHeader.alg);
            if (keyObject instanceof Uint8Array || keyObject.type !== 'public') {
                throw new JWKSInvalid('JSON Web Key Set members must be public keys');
            }
            cached[joseHeader.alg] = keyObject;
        }
        return cached[joseHeader.alg];
    }
}
class JWT {
    #projectId;
    #clientEmail;
    #privateKeyId;
    #privateKeyString;
    #privateKey;
    constructor(projectId, clientEmail, privateKeyId, privateKeyString){
        this.#projectId = projectId;
        this.#clientEmail = clientEmail;
        this.#privateKeyId = privateKeyId;
        this.#privateKeyString = privateKeyString;
    }
    static fromJSON(json) {
        if (!json) {
            throw new Error("Must pass in a JSON object containing the service account auth settings.");
        }
        if (!json.client_email) {
            throw new Error("The incoming JSON object does not contain a client_email field");
        }
        if (!json.private_key) {
            throw new Error("The incoming JSON object does not contain a private_key field");
        }
        return new JWT(json.project_id, json.client_email, json.private_key_id, json.private_key);
    }
    get projectId() {
        return this.#projectId;
    }
    async getRequestHeaders(url) {
        const aud = new URL(url).origin + "/";
        const jwt = await this.#getJWT(aud);
        return {
            "Authorization": `Bearer ${jwt}`
        };
    }
    #getPrivateKey() {
        if (!this.#privateKey) {
            this.#privateKey = importPKCS8(this.#privateKeyString, "RS256");
        }
        return this.#privateKey;
    }
    async #getJWT(aud) {
        const key = await this.#getPrivateKey();
        return new SignJWT({
            aud
        }).setProtectedHeader({
            alg: "RS256",
            kid: this.#privateKeyId
        }).setIssuer(this.#clientEmail).setSubject(this.#clientEmail).setIssuedAt().setExpirationTime("1h").sign(key);
    }
}
const EXTERNAL_ACCOUNT_TYPE = "external_account";
class GoogleAuth {
    fromJSON(json) {
        let client;
        if (!json) {
            throw new Error("Must pass in a JSON object containing the Google auth settings.");
        }
        if (json.type === "authorized_user") {
            throw new Error("TBD");
        } else if (json.type === EXTERNAL_ACCOUNT_TYPE) {
            throw new Error("TBD");
        } else {
            client = JWT.fromJSON(json);
        }
        return client;
    }
    async getApplicationDefault() {
        let client = null;
        client = await this.#tryGetApplicationCredentialsFromEnvironmentVariable();
        if (client !== null) {
            return {
                credential: client,
                projectId: client.projectId ?? null
            };
        }
        throw new Error("Could not load the default credentials. Browse to https://cloud.google.com/docs/authentication/getting-started for more information.");
    }
    async #tryGetApplicationCredentialsFromEnvironmentVariable() {
        const filePath = getEnv("GOOGLE_APPLICATION_CREDENTIALS") || getEnv("google_application_credentials");
        if (filePath === undefined || filePath.length === 0) {
            return null;
        }
        try {
            const text = await Deno.readTextFile(filePath);
            return this.fromJSON(JSON.parse(text));
        } catch (e) {
            if (e instanceof Deno.errors.NotFound || e instanceof Deno.errors.PermissionDenied) {
                throw new Error(`Unable to read the credential file specified by the GOOGLE_APPLICATION_CREDENTIALS environment variable.`, {
                    cause: e
                });
            }
            throw e;
        }
    }
}
new GoogleAuth();
async function request(url, opts) {
    const headers = await opts.client?.getRequestHeaders(url) ?? {};
    const resp = await fetch(url, {
        headers: {
            "accept": "application/json",
            "content-type": "application/json",
            ...headers
        },
        body: opts.body,
        method: opts.method
    });
    if (resp.status >= 400) {
        if (resp.headers.get("content-type")?.includes("application/json")) {
            const body = await resp.json();
            throw new GoogleApiError(body.error.code, body.error.message, body.error.details);
        } else {
            const body = await resp.text();
            throw new GoogleApiError(resp.status, body, undefined);
        }
    }
    return await resp.json();
}
class GoogleApiError extends Error {
    code;
    details;
    constructor(code, message, details){
        super(`${code}: ${message}`);
        this.name = "GoogleApiError";
        this.code = code;
        this.details = details;
    }
}
class Sheets {
    #client;
    #baseUrl;
    constructor(client, baseUrl = "https://sheets.googleapis.com/"){
        this.#client = client;
        this.#baseUrl = baseUrl;
    }
    async spreadsheetsBatchUpdate(spreadsheetId, req) {
        req = serializeBatchUpdateSpreadsheetRequest(req);
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}:batchUpdate`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return deserializeBatchUpdateSpreadsheetResponse(data);
    }
    async spreadsheetsCreate(req) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsDeveloperMetadataGet(metadataId, spreadsheetId) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/developerMetadata/${metadataId}`);
        const data = await request(url.href, {
            client: this.#client,
            method: "GET"
        });
        return data;
    }
    async spreadsheetsDeveloperMetadataSearch(spreadsheetId, req) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/developerMetadata:search`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsGet(spreadsheetId, opts = {}) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}`);
        if (opts.includeGridData !== undefined) {
            url.searchParams.append("includeGridData", String(opts.includeGridData));
        }
        if (opts.ranges !== undefined) {
            url.searchParams.append("ranges", String(opts.ranges));
        }
        const data = await request(url.href, {
            client: this.#client,
            method: "GET"
        });
        return data;
    }
    async spreadsheetsGetByDataFilter(spreadsheetId, req) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}:getByDataFilter`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsSheetsCopyTo(sheetId, spreadsheetId, req) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/sheets/${sheetId}:copyTo`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsValuesAppend(range, spreadsheetId, req, opts = {}) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/values/${range}:append`);
        if (opts.includeValuesInResponse !== undefined) {
            url.searchParams.append("includeValuesInResponse", String(opts.includeValuesInResponse));
        }
        if (opts.insertDataOption !== undefined) {
            url.searchParams.append("insertDataOption", String(opts.insertDataOption));
        }
        if (opts.responseDateTimeRenderOption !== undefined) {
            url.searchParams.append("responseDateTimeRenderOption", String(opts.responseDateTimeRenderOption));
        }
        if (opts.responseValueRenderOption !== undefined) {
            url.searchParams.append("responseValueRenderOption", String(opts.responseValueRenderOption));
        }
        if (opts.valueInputOption !== undefined) {
            url.searchParams.append("valueInputOption", String(opts.valueInputOption));
        }
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsValuesBatchClear(spreadsheetId, req) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/values:batchClear`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsValuesBatchClearByDataFilter(spreadsheetId, req) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/values:batchClearByDataFilter`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsValuesBatchGet(spreadsheetId, opts = {}) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/values:batchGet`);
        if (opts.dateTimeRenderOption !== undefined) {
            url.searchParams.append("dateTimeRenderOption", String(opts.dateTimeRenderOption));
        }
        if (opts.majorDimension !== undefined) {
            url.searchParams.append("majorDimension", String(opts.majorDimension));
        }
        if (opts.ranges !== undefined) {
            url.searchParams.append("ranges", String(opts.ranges));
        }
        if (opts.valueRenderOption !== undefined) {
            url.searchParams.append("valueRenderOption", String(opts.valueRenderOption));
        }
        const data = await request(url.href, {
            client: this.#client,
            method: "GET"
        });
        return data;
    }
    async spreadsheetsValuesBatchGetByDataFilter(spreadsheetId, req) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/values:batchGetByDataFilter`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsValuesBatchUpdate(spreadsheetId, req) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/values:batchUpdate`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsValuesBatchUpdateByDataFilter(spreadsheetId, req) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/values:batchUpdateByDataFilter`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsValuesClear(range, spreadsheetId, req) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/values/${range}:clear`);
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "POST",
            body
        });
        return data;
    }
    async spreadsheetsValuesGet(range, spreadsheetId, opts = {}) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/values/${range}`);
        if (opts.dateTimeRenderOption !== undefined) {
            url.searchParams.append("dateTimeRenderOption", String(opts.dateTimeRenderOption));
        }
        if (opts.majorDimension !== undefined) {
            url.searchParams.append("majorDimension", String(opts.majorDimension));
        }
        if (opts.valueRenderOption !== undefined) {
            url.searchParams.append("valueRenderOption", String(opts.valueRenderOption));
        }
        const data = await request(url.href, {
            client: this.#client,
            method: "GET"
        });
        return data;
    }
    async spreadsheetsValuesUpdate(range, spreadsheetId, req, opts = {}) {
        const url = new URL(`${this.#baseUrl}v4/spreadsheets/${spreadsheetId}/values/${range}`);
        if (opts.includeValuesInResponse !== undefined) {
            url.searchParams.append("includeValuesInResponse", String(opts.includeValuesInResponse));
        }
        if (opts.responseDateTimeRenderOption !== undefined) {
            url.searchParams.append("responseDateTimeRenderOption", String(opts.responseDateTimeRenderOption));
        }
        if (opts.responseValueRenderOption !== undefined) {
            url.searchParams.append("responseValueRenderOption", String(opts.responseValueRenderOption));
        }
        if (opts.valueInputOption !== undefined) {
            url.searchParams.append("valueInputOption", String(opts.valueInputOption));
        }
        const body = JSON.stringify(req);
        const data = await request(url.href, {
            client: this.#client,
            method: "PUT",
            body
        });
        return data;
    }
}
function deserializeAddDataSourceResponse(data) {
    return {
        ...data,
        dataExecutionStatus: data["dataExecutionStatus"] !== undefined ? deserializeDataExecutionStatus(data["dataExecutionStatus"]) : undefined
    };
}
function serializeAppendCellsRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeBatchUpdateSpreadsheetRequest(data) {
    return {
        ...data,
        requests: data["requests"] !== undefined ? data["requests"].map((item)=>serializeRequest(item)) : undefined
    };
}
function deserializeBatchUpdateSpreadsheetResponse(data) {
    return {
        ...data,
        replies: data["replies"] !== undefined ? data["replies"].map((item)=>deserializeResponse(item)) : undefined
    };
}
function deserializeDataExecutionStatus(data) {
    return {
        ...data,
        lastRefreshTime: data["lastRefreshTime"] !== undefined ? new Date(data["lastRefreshTime"]) : undefined
    };
}
function deserializeRefreshDataSourceObjectExecutionStatus(data) {
    return {
        ...data,
        dataExecutionStatus: data["dataExecutionStatus"] !== undefined ? deserializeDataExecutionStatus(data["dataExecutionStatus"]) : undefined
    };
}
function deserializeRefreshDataSourceResponse(data) {
    return {
        ...data,
        statuses: data["statuses"] !== undefined ? data["statuses"].map((item)=>deserializeRefreshDataSourceObjectExecutionStatus(item)) : undefined
    };
}
function serializeRepeatCellRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeRequest(data) {
    return {
        ...data,
        appendCells: data["appendCells"] !== undefined ? serializeAppendCellsRequest(data["appendCells"]) : undefined,
        repeatCell: data["repeatCell"] !== undefined ? serializeRepeatCellRequest(data["repeatCell"]) : undefined,
        updateBanding: data["updateBanding"] !== undefined ? serializeUpdateBandingRequest(data["updateBanding"]) : undefined,
        updateCells: data["updateCells"] !== undefined ? serializeUpdateCellsRequest(data["updateCells"]) : undefined,
        updateDataSource: data["updateDataSource"] !== undefined ? serializeUpdateDataSourceRequest(data["updateDataSource"]) : undefined,
        updateDeveloperMetadata: data["updateDeveloperMetadata"] !== undefined ? serializeUpdateDeveloperMetadataRequest(data["updateDeveloperMetadata"]) : undefined,
        updateDimensionGroup: data["updateDimensionGroup"] !== undefined ? serializeUpdateDimensionGroupRequest(data["updateDimensionGroup"]) : undefined,
        updateDimensionProperties: data["updateDimensionProperties"] !== undefined ? serializeUpdateDimensionPropertiesRequest(data["updateDimensionProperties"]) : undefined,
        updateEmbeddedObjectBorder: data["updateEmbeddedObjectBorder"] !== undefined ? serializeUpdateEmbeddedObjectBorderRequest(data["updateEmbeddedObjectBorder"]) : undefined,
        updateEmbeddedObjectPosition: data["updateEmbeddedObjectPosition"] !== undefined ? serializeUpdateEmbeddedObjectPositionRequest(data["updateEmbeddedObjectPosition"]) : undefined,
        updateFilterView: data["updateFilterView"] !== undefined ? serializeUpdateFilterViewRequest(data["updateFilterView"]) : undefined,
        updateNamedRange: data["updateNamedRange"] !== undefined ? serializeUpdateNamedRangeRequest(data["updateNamedRange"]) : undefined,
        updateProtectedRange: data["updateProtectedRange"] !== undefined ? serializeUpdateProtectedRangeRequest(data["updateProtectedRange"]) : undefined,
        updateSheetProperties: data["updateSheetProperties"] !== undefined ? serializeUpdateSheetPropertiesRequest(data["updateSheetProperties"]) : undefined,
        updateSlicerSpec: data["updateSlicerSpec"] !== undefined ? serializeUpdateSlicerSpecRequest(data["updateSlicerSpec"]) : undefined,
        updateSpreadsheetProperties: data["updateSpreadsheetProperties"] !== undefined ? serializeUpdateSpreadsheetPropertiesRequest(data["updateSpreadsheetProperties"]) : undefined
    };
}
function deserializeResponse(data) {
    return {
        ...data,
        addDataSource: data["addDataSource"] !== undefined ? deserializeAddDataSourceResponse(data["addDataSource"]) : undefined,
        refreshDataSource: data["refreshDataSource"] !== undefined ? deserializeRefreshDataSourceResponse(data["refreshDataSource"]) : undefined,
        updateDataSource: data["updateDataSource"] !== undefined ? deserializeUpdateDataSourceResponse(data["updateDataSource"]) : undefined
    };
}
function serializeUpdateBandingRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateCellsRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateDataSourceRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function deserializeUpdateDataSourceResponse(data) {
    return {
        ...data,
        dataExecutionStatus: data["dataExecutionStatus"] !== undefined ? deserializeDataExecutionStatus(data["dataExecutionStatus"]) : undefined
    };
}
function serializeUpdateDeveloperMetadataRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateDimensionGroupRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateDimensionPropertiesRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateEmbeddedObjectBorderRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateEmbeddedObjectPositionRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateFilterViewRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateNamedRangeRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateProtectedRangeRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateSheetPropertiesRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateSlicerSpecRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
function serializeUpdateSpreadsheetPropertiesRequest(data) {
    return {
        ...data,
        fields: data["fields"] !== undefined ? data["fields"] : undefined
    };
}
new Sheets();
console.log("pepe");
