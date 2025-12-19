use translators::{GoogleTranslator, Translator};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ==========================================
    // إعدادات البرنامج
    // ==========================================
    let input_filename = "source.txt";  // اسم ملف المصدر
    let output_filename = "output.po";  // اسم ملف الناتج (تنسيق PO)
    let target_lang = "ar";             // اللغة الهدف (عربي)
    let delay_ms = 500;                 // التأخير بالمللي ثانية (للحماية من الحظر)
    // ==========================================

    println!("--- بدء برنامج الترجمة الآلية ---");
    
    // 1. إعداد المترجم
    let translator = GoogleTranslator::default();

    // 2. محاولة فتح ملف المصدر
    let input_file = match File::open(input_filename) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("خطأ: لم يتم العثور على الملف '{}'. تأكد من وجوده بجانب مجلد examples.", input_filename);
            return Ok(());
        }
    };
    let reader = BufReader::new(input_file);

    // 3. إنشاء ملف الناتج
    let mut output_file = File::create(output_filename)?;

    println!("جاري قراءة '{}' والترجمة إلى '{}'...", input_filename, output_filename);

    // 4. حلقة تكرار لقراءة الأسطر
    for (index, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let original_text = line.trim();

        // تجاوز الأسطر الفارغة
        if original_text.is_empty() {
            continue;
        }

        // طباعة توضيحية في الطرفية (Progress)
        print!("سطر {}: جاري الترجمة... ", index + 1);
        std::io::stdout().flush()?; // لإظهار النص فوراً

        // محاولة الترجمة
        // الوسيط الثاني "" يعني اكتشاف اللغة تلقائياً
        match translator.translate_async(original_text, "", target_lang).await {
            Ok(translated_text) => {
                println!("تم ✓");

                // --- تنسيق ملف PO ---
                // يجب وضع علامة \ قبل أي علامة تنصيص داخل النص (Escaping)
                let safe_original = original_text.replace("\"", "\\\"");
                let safe_translated = translated_text.replace("\"", "\\\"");

                // كتابة msgid (النص الأصلي)
                writeln!(output_file, "msgid \"{}\"", safe_original)?;
                
                // كتابة msgstr (النص المترجم)
                writeln!(output_file, "msgstr \"{}\"", safe_translated)?;
                
                // سطر فارغ للفصل بين المدخلات
                writeln!(output_file, "")?;
            }
            Err(e) => {
                println!("فشل X");
                eprintln!("  -> السبب: {}", e);
                
                // في حالة الفشل، نكتب النص الأصلي ونترك الترجمة فارغة أو نكرر الأصلي
                let safe_original = original_text.replace("\"", "\\\"");
                writeln!(output_file, "# خطأ في الترجمة هنا", )?;
                writeln!(output_file, "msgid \"{}\"", safe_original)?;
                writeln!(output_file, "msgstr \"\"", )?;
                writeln!(output_file, "")?;
            }
        }

        // 5. الانتظار (Sleep) لتجنب الحظر من جوجل
        sleep(Duration::from_millis(delay_ms)).await;
    }

    println!("--------------------------------------------------");
    println!("تمت العملية بنجاح! الملف الناتج: {}", output_filename);
    
    Ok(())
}
