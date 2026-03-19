# Swiftness Data Model

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Source:** Swiftness (בורסה לניירות ערך בישראל - Israeli Pension Clearing House) Excel Export

## Overview

This document defines the data model for Swiftness Excel file imports. Swiftness exports contain Israeli pension, insurance, and savings account information with validity dates indicating when each instrument's data is accurate.

## File Format

- **Format:** Legacy Microsoft Excel (.xls)
- **Encoding:** Code page 1255 (Hebrew)
- **Sheets:** 3 sheets with different data types
- **Language:** Hebrew (right-to-left text)

## Sheet 1: Insurance Coverage (כיסויים ביטוחיים)

**Purpose:** Lists insurance policy coverage details

**Rows:** 15 (including header)
**Columns:** 7

### Column Mapping

| Column | Hebrew Name | English Translation | Data Type | Notes |
|--------|-------------|-------------------|----------|-------|
| 1 | סוג הכיסוי הביטוחי | Coverage Type | Text | e.g., "כיסוי למקרה מוות" (Death Coverage) |
| 2 | שם התוכנית | Plan Name | Text | e.g., "אור 1 - ביטוח חיים" |
| 3 | שם חברה מנהלת/מבטחת | Managing/Insuring Company | Text | e.g., "מגדל", "כלל" |
| 4 | מקבל התשלום | Payment Recipient | Text | e.g., "כלל המוטבים" (All Beneficiaries) |
| 5 | סכום חד פעמי | One-Time Amount | Number | ILS currency |
| 6 | קצבה חודשית | Monthly Benefit | Number | ILS currency |
| 7 | מס' פוליסה/חשבון | Policy/Account Number | Text | **Unique Identifier** |

### Validity Date

**⚠️ No validity date column in this sheet**

This sheet contains static reference data about insurance coverage. The data validity is determined by the file modification date or should be cross-referenced with Sheet 3's validity dates using the policy number.

### Data Model

```python
@dataclass
class InsuranceCoverage:
    coverage_type: str          # סוג הכיסוי הביטוחי
    plan_name: str             # שם התוכנית
    company: str               # שם חברה מנהלת/מבטחת
    payment_recipient: str     # מקבל התשלום
    one_time_amount: float     # סכום חד פעמי (ILS)
    monthly_benefit: float     # קצבה חודשית (ILS)
    policy_number: str         # מס' פוליסה/חשבון (Unique ID)
```

## Sheet 2: Deposit Tracking (מעקב הפקדות)

**Purpose:** Tracks monthly deposits to pension/savings funds

**Rows:** 45 (including header)
**Columns:** 9

### Column Mapping

| Column | Hebrew Name | English Translation | Data Type | Notes |
|--------|-------------|-------------------|----------|-------|
| 1 | סוג מוצר | Product Type | Text | e.g., "קרן השתלמות" (Training Fund) |
| 2 | שם חברה מנהלת | Managing Company | Text | e.g., "כלל פנסיה וגמל בע"מ" |
| 3 | מספר פוליסה | Policy Number | Text | **Unique Identifier** |
| 4 | **תאריך ערך** | **Value Date** | **Date (Text)** | **✅ VALIDITY DATE** - Format: M/D/YYYY |
| 5 | חודש שכר | Salary Month | Date (Text) | Format: M/D/YYYY |
| 6 | שם מעסיק | Employer Name | Text | e.g., "אינפינידט בע"מ" |
| 7 | הפקדות עובד | Employee Deposits | Number | ILS currency |
| 8 | הפקדות מעסיק | Employer Deposits | Number | ILS currency |
| 9 | הפקדות מעסיק לפיצויים | Employer Severance Deposits | Number | ILS currency |

### Validity Date

**Column 4: "תאריך ערך" (Value Date)**

- **Format:** Text dates in M/D/YYYY format (e.g., "7/11/2024", "8/12/2024")
- **Meaning:** The date up to which the deposit record data is accurate
- **Usage:** Each row represents a deposit transaction valid up to the value date
- **Example:** A row with value date "7/11/2024" means the deposit data is accurate as of November 7, 2024

### Data Model

```python
@dataclass
class DepositRecord:
    product_type: str          # סוג מוצר
    company: str               # שם חברה מנהלת
    policy_number: str         # מספר פוליסה (Unique ID)
    value_date: datetime       # תאריך ערך (VALIDITY DATE)
    salary_month: datetime       # חודש שכר
    employer_name: str         # שם מעסיק
    employee_deposit: float    # הפקדות עובד (ILS)
    employer_deposit: float    # הפקדות מעסיק (ILS)
    employer_severance: float  # הפקדות מעסיק לפיצויים (ILS)
```

## Sheet 3: Product Details (פרטי המוצרים שלי)

**Purpose:** Detailed information about each financial product/policy

**Rows:** 12 (including header)
**Columns:** 30

### Key Columns

| Column | Hebrew Name | English Translation | Data Type | Notes |
|--------|-------------|-------------------|----------|-------|
| 1 | שם מוצר | Product Name | Text | Full product description |
| 2 | שם חברה מנהלת | Managing Company | Text | Company name |
| 3 | מספר פוליסה | Policy Number | Text | **Unique Identifier** |
| 4 | סטטוס | Status | Text | e.g., "פעיל" (Active), "לא פעיל" (Inactive) |
| 5 | סך הכל חיסכון | Total Savings | Number | ILS currency |
| 6 | תחנת משיכה קרובה | Next Withdrawal Date | Date (Text) | Format: M/D/YYYY |
| 7-10 | Various retirement projections | Numbers | ILS currency | Projected savings/benefits |
| 11-13 | Management fees | Numbers | Percentages | Fee rates |
| 14 | תשואה מתחילת השנה | YTD Return | Number | Percentage |
| 15-16 | Deposits (employee/employer) | Numbers | ILS currency | Deposit amounts |
| 17-20 | Beneficiary information | Text/Numbers | Various | Beneficiary details |
| 21-22 | Disability insurance amounts | Numbers | ILS currency | Monthly/one-time |
| 23 | תאריך הצטרפות לראשונה | First Join Date | Date (Text) | Format: M/D/YYYY |
| 24-25 | Beneficiaries | Text | Various | Beneficiary names |
| 26-27 | Death insurance amounts | Numbers | ILS currency | Monthly/one-time |
| 28 | סוג מוצר | Product Type | Text | Product category |
| 29 | תאריך פתיחת תוכנית | Plan Opening Date | Date (Text) | Format: M/D/YYYY |
| 30 | **תאריך נכונות נתונים** | **Data Accuracy Date** | **Date (Text)** | **✅ VALIDITY DATE** - Format: M/D/YYYY |

### Validity Date

**Column 30: "תאריך נכונות נתונים" (Data Accuracy Date)**

- **Format:** Text dates in M/D/YYYY format (e.g., "9/30/2025")
- **Meaning:** The date up to which all product data in this row is accurate
- **Usage:** This is the **primary validity date** for product-level data
- **Example:** A product with data accuracy date "9/30/2025" means all financial data (savings, projections, etc.) is accurate as of September 30, 2025
- **Important:** This date applies to ALL columns in the row (savings amounts, projections, fees, etc.)

### Data Model

```python
@dataclass
class ProductDetails:
    product_name: str          # שם מוצר
    company: str               # שם חברה מנהלת
    policy_number: str         # מספר פוליסה (Unique ID)
    status: str                # סטטוס (Active/Inactive)
    total_savings: float       # סך הכל חיסכון (ILS)
    next_withdrawal_date: Optional[datetime]  # תחנת משיכה קרובה
    projected_savings_no_premiums: float      # חיסכון צפוי לגיל פרישה לא כולל פרמיות
    monthly_benefit_no_premiums: float         # קיצבה חודשית לגיל פרישה לא כולל פרמיות
    projected_savings: float   # חיסכון צפוי לגיל פרישה
    monthly_benefit: float     # קיצבה חודשית לגיל פרישה
    expected_pension_rate: float  # שיעור פנסיה זקנה צפויה
    management_fee_on_deposits: float  # שיעור דמי ניהול מהפקדות
    management_fee_annual: float       # שיעור דמי ניהול שנתי מחיסכון צבור
    ytd_return: float         # תשואה מתחילת השנה
    employee_deposits: float  # הפקדות חוסך
    employer_deposits: float  # הפקדות מעסיק
    # ... beneficiary fields ...
    first_join_date: datetime  # תאריך הצטרפות לראשונה
    plan_opening_date: datetime  # תאריך פתיחת תוכנית
    data_accuracy_date: datetime  # תאריך נכונות נתונים (VALIDITY DATE)
```

## Validity Date Summary

### Sheet 1: Insurance Coverage

- **No validity date column**
- Use file modification date or cross-reference with Sheet 3

### Sheet 2: Deposit Tracking

- **Column 4: "תאריך ערך" (Value Date)**
- Each deposit record has its own validity date
- Format: M/D/YYYY (text)

### Sheet 3: Product Details

- **Column 30: "תאריך נכונות נתונים" (Data Accuracy Date)**
- One validity date per product (applies to all product data)
- Format: M/D/YYYY (text)
- **This is the primary validity date for product-level updates**

## Data Relationships

### Unique Identifiers

- **Policy Number (מספר פוליסה)**: Used across all sheets to link related records
- Sheet 1: Column 7
- Sheet 2: Column 3
- Sheet 3: Column 3

### Cross-Reference Strategy

1. Use policy number to link records across sheets
2. Sheet 3's validity date (Column 30) applies to product-level data
3. Sheet 2's validity dates (Column 4) apply to individual deposit records
4. Sheet 1 can be linked via policy number but has no validity date

## Date Format Handling

### Date Parsing

- All dates are stored as **text** in M/D/YYYY format
- Examples: "7/11/2024", "9/30/2025", "12/26/2022"
- **Important:** Month comes first (US format), not day-first (Israeli format)
- Parse using: `datetime.strptime(date_str, "%m/%d/%Y")`

### Date Validation

- Check if validity date is in the future (data is still valid)
- Check if validity date has passed (data may be stale)
- Compare validity dates when updating existing records

## Update Logic

### When to Update

1. **New file imported:** Compare validity dates with existing data
2. **Validity date check:** If new data has later validity date, update
3. **Policy matching:** Match by policy number across sheets
4. **Conflict resolution:** Always prefer data with later validity date

### Update Rules

- **Sheet 2 (Deposits):** Each row is a separate transaction - add new rows, don't overwrite
- **Sheet 3 (Products):** Update entire product record if validity date is newer
- **Sheet 1 (Coverage):** Update if policy number matches and file is newer

## Currency

- All monetary values are in **Israeli Shekels (ILS)**
- Convert to USD for unified portfolio view (see Investment Strategy Framework)
- Conversion rate should be fetched from currency API or user-provided

## Encoding

- File uses **Code page 1255** (Hebrew Windows encoding)
- Text fields contain Hebrew characters (right-to-left)
- Ensure proper encoding when reading: `xlrd.open_workbook(filename, encoding_override='cp1255')`

## Integration Notes

- This data integrates with the Investment Strategy Framework
- Positions should be converted to USD for portfolio aggregation
- Validity dates determine when data refresh is needed
- Policy numbers serve as unique identifiers for position matching
