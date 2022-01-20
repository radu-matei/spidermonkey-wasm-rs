#include "api.h"

std::unique_ptr<JSClass> getDefaultGlobalClass() {
  const JSClass defaultGlobal = { 
    "Global",
    JSCLASS_GLOBAL_FLAGS,
    &JS::DefaultGlobalClassOps
  };

  return std::make_unique<JSClass>(defaultGlobal);
}

JS::RealmOptions* makeDefaultRealmOptions() {
  return new JS::RealmOptions();
}

std::unique_ptr<JS::OwningCompileOptions> NewOwningCompileOptions(JSContext* context, const CompileOptionsParams &opts) {
  JS::CompileOptions jsOpts(context);

  if (opts.force_full_parse) {
    jsOpts.setForceFullParse();
  }

  // TODO: Ideally, this line should use `c_str` instead of
  // `data`; the const *char yielded by `data` is not null-terminated
  jsOpts.setFileAndLine(opts.file.data(), opts.lineno);

  auto owningOpts = std::make_unique<JS::OwningCompileOptions>(context);

  // TODO: Handle the case where `copy` returns `false`
  owningOpts->copy(context, jsOpts);

  return owningOpts;
}

bool InitDefaultSelfHostedCode(JSContext* context) {
  return JS::InitSelfHostedCode(context);
}

std::unique_ptr<Utf8UnitSourceText> MakeUtf8UnitSourceText() {
  return std::make_unique<Utf8UnitSourceText>();
}

bool InitUtf8UnitSourceText(JSContext* context, Utf8UnitSourceText& src, rust::Str units, size_t length, JS::SourceOwnership ownership) {
  return src.init(context, units.data(), length, ownership);
}

bool Utf8SourceEvaluate(JSContext* context, const JS::OwningCompileOptions& opts, Utf8UnitSourceText& src, JS::MutableHandle<JS::Value> rval) {
  return JS::Evaluate(context, opts, src, rval);
}