import Foundation
import Security

protocol CredentialStore {
    func load() throws -> String?
    func save(_ token: String) throws
    func restore(_ token: String?) throws
}

struct KeychainCredentialStore: CredentialStore {
    static let defaultService = "com.nexthrum.gotify"
    static let defaultAccount = "client-token"

    private let service: String
    private let account: String

    /// Service/account are injectable so tests never touch the real token item.
    init(service: String = KeychainCredentialStore.defaultService,
         account: String = KeychainCredentialStore.defaultAccount) {
        self.service = service
        self.account = account
    }

    func load() throws -> String? {
        var query = baseQuery
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne
        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        if status == errSecItemNotFound { return nil }
        guard status == errSecSuccess else { throw keychainError(status) }
        guard let data = result as? Data, let token = String(data: data, encoding: .utf8) else {
            throw NSError(domain: "NexthrumKeychain", code: -1, userInfo: [NSLocalizedDescriptionKey: "Stored credential is not valid UTF-8"])
        }
        return token
    }

    func save(_ token: String) throws {
        let data = Data(token.utf8)
        let status = SecItemUpdate(baseQuery as CFDictionary, [kSecValueData as String: data] as CFDictionary)
        if status == errSecItemNotFound {
            var query = baseQuery
            query[kSecValueData as String] = data
            let addStatus = SecItemAdd(query as CFDictionary, nil)
            guard addStatus == errSecSuccess else { throw keychainError(addStatus) }
        } else if status != errSecSuccess {
            throw keychainError(status)
        }
    }

    func restore(_ token: String?) throws {
        guard let token else {
            let status = SecItemDelete(baseQuery as CFDictionary)
            guard status == errSecSuccess || status == errSecItemNotFound else {
                throw keychainError(status)
            }
            return
        }
        try save(token)
    }

    private var baseQuery: [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
    }

    private func keychainError(_ status: OSStatus) -> NSError {
        NSError(domain: NSOSStatusErrorDomain, code: Int(status))
    }
}

