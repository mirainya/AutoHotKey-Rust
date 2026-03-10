import * as monaco from 'monaco-editor'

export function setupRhaiLanguage() {
  // 注册 Rhai 语言
  monaco.languages.register({ id: 'rhai' })

  // 配置语法高亮
  monaco.languages.setMonarchTokensProvider('rhai', {
    keywords: [
      'if', 'else', 'while', 'loop', 'for', 'in', 'break', 'continue',
      'return', 'fn', 'let', 'const', 'true', 'false', 'import', 'export'
    ],

    operators: [
      '=', '>', '<', '!', '~', '?', ':',
      '==', '<=', '>=', '!=', '&&', '||', '++', '--',
      '+', '-', '*', '/', '&', '|', '^', '%', '<<',
      '>>', '>>>', '+=', '-=', '*=', '/=', '&=', '|=',
      '^=', '%=', '<<=', '>>=', '>>>='
    ],

    tokenizer: {
      root: [
        [/[a-zA-Z_]\w*/, {
          cases: {
            '@keywords': 'keyword',
            '@default': 'identifier'
          }
        }],
        [/"([^"\\]|\\.)*$/, 'string.invalid'],
        [/"/, 'string', '@string'],
        [/\d+/, 'number'],
        [/\/\/.*$/, 'comment'],
      ],

      string: [
        [/[^\\"]+/, 'string'],
        [/"/, 'string', '@pop']
      ]
    }
  })

  // 配置代码补全
  monaco.languages.registerCompletionItemProvider('rhai', {
    provideCompletionItems: () => {
      const suggestions = [
        {
          label: '截图',
          kind: monaco.languages.CompletionItemKind.Function,
          insertText: '截图()',
          detail: '截取屏幕并返回图片路径'
        },
        {
          label: '移动鼠标',
          kind: monaco.languages.CompletionItemKind.Function,
          insertText: '移动鼠标(${1:x}, ${2:y})',
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          detail: '移动鼠标到指定坐标'
        },
        {
          label: '输入文本',
          kind: monaco.languages.CompletionItemKind.Function,
          insertText: '输入文本("${1:text}")',
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          detail: '输入指定文本'
        },
        {
          label: '等待',
          kind: monaco.languages.CompletionItemKind.Function,
          insertText: '等待(${1:1000})',
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          detail: '等待指定毫秒数'
        },
        {
          label: 'print',
          kind: monaco.languages.CompletionItemKind.Function,
          insertText: 'print("${1:message}")',
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
          detail: '打印消息到控制台'
        }
      ]
      return { suggestions }
    }
  })
}
