#pragma once
#include <cstdint>
typedef void Il2CppClass; typedef void Il2CppType; typedef void Il2CppImage;
typedef void Il2CppAssembly; typedef void Il2CppDomain; typedef void Il2CppString; typedef void Il2CppObject;
struct Il2CppMethodInfo { void* methodPointer; void* invoker_method; const char* name; Il2CppClass* klass; const Il2CppType* return_type; void* parameters; };
typedef Il2CppDomain*    (*il2cpp_domain_get_t)();
typedef Il2CppAssembly** (*il2cpp_domain_get_assemblies_t)(Il2CppDomain*, size_t*);
typedef Il2CppImage*     (*il2cpp_assembly_get_image_t)(Il2CppAssembly*);
typedef const char*      (*il2cpp_image_get_name_t)(Il2CppImage*);
typedef Il2CppClass*     (*il2cpp_class_from_name_t)(Il2CppImage*, const char*, const char*);
typedef Il2CppMethodInfo*(*il2cpp_class_get_method_from_name_t)(Il2CppClass*, const char*, int);
typedef void             (*il2cpp_runtime_class_init_t)(Il2CppClass*);
typedef Il2CppString*    (*il2cpp_string_new_t)(const char*);
typedef size_t           (*il2cpp_class_instance_size_t)(Il2CppClass*);
struct Il2CppApi {
    il2cpp_domain_get_t domain_get; il2cpp_domain_get_assemblies_t domain_get_assemblies;
    il2cpp_assembly_get_image_t assembly_get_image; il2cpp_image_get_name_t image_get_name;
    il2cpp_class_from_name_t class_from_name; il2cpp_class_get_method_from_name_t method_from_name;
    il2cpp_runtime_class_init_t runtime_class_init; il2cpp_string_new_t string_new;
    il2cpp_class_instance_size_t class_instance_size;
};
extern Il2CppApi g_il2cpp;
