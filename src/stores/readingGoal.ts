import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { useDynamicConfig } from "@/composables/useDynamicConfig";

export type GoalType = "minutes" | "words";

export interface ReadingGoalState {
  goalMinutes: number;
  goalType: GoalType;
  goalWords: number;
  todayMinutes: number;
  todayWords: number;
  streakDays: number;
  lastReadDate: string;
  streakHistory: string[];
}

interface DailyRecord {
  date: string;
  minutes: number;
  words: number;
}

interface ReadingGoalConfig extends ReadingGoalState {
  dailyRecords: DailyRecord[];
}

function getTodayString(): string {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function getYesterdayString(): string {
  const now = new Date();
  now.setDate(now.getDate() - 1);
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export const useReadingGoalStore = defineStore("readingGoal", () => {
  const config = useDynamicConfig<ReadingGoalConfig>({
    namespace: "reading-goal",
    version: 1,
    defaults: (): ReadingGoalConfig => ({
      goalMinutes: 30,
      goalType: "minutes",
      goalWords: 5000,
      todayMinutes: 0,
      todayWords: 0,
      streakDays: 0,
      lastReadDate: "",
      streakHistory: [],
      dailyRecords: [],
    }),
  });

  const goalMinutes = computed(() => config.state.goalMinutes);
  const goalType = computed(() => config.state.goalType);
  const goalWords = computed(() => config.state.goalWords);
  // 自然日校验：跨日未触发 checkTodayReset 时按 0 暴露，
  // 避免昨天累计的 todayMinutes 被今天的 UI 当成"今日进度"显示。
  const todayMinutes = computed(() =>
    config.state.lastReadDate === getTodayString() ? config.state.todayMinutes : 0,
  );
  const todayWords = computed(() =>
    config.state.lastReadDate === getTodayString() ? config.state.todayWords : 0,
  );
  const streakDays = computed(() => config.state.streakDays);
  const lastReadDate = computed(() => config.state.lastReadDate);
  const streakHistory = computed(() => config.state.streakHistory);

  const dailyRecords = computed(() => config.state.dailyRecords);

  const progressPercentage = computed(() => {
    if (goalType.value === "minutes") {
      if (goalMinutes.value === 0) return 0;
      return Math.min(100, (todayMinutes.value / goalMinutes.value) * 100);
    } else {
      if (goalWords.value === 0) return 0;
      return Math.min(100, (todayWords.value / goalWords.value) * 100);
    }
  });

  function checkTodayReset() {
    const today = getTodayString();
    if (lastReadDate.value && lastReadDate.value !== today) {
      config.state.todayMinutes = 0;
      config.state.todayWords = 0;
    }
  }

  async function setGoalType(type: GoalType) {
    config.state.goalType = type;
  }

  async function setGoalMinutes(minutes: number) {
    config.state.goalMinutes = Math.max(0, minutes);
  }

  async function setGoalWords(words: number) {
    config.state.goalWords = Math.max(0, words);
  }

  async function addReadingMinutes(mins: number): Promise<void> {
    const today = getTodayString();
    checkTodayReset();

    config.state.todayMinutes += mins;
    config.state.lastReadDate = today;

    const existingRecord = config.state.dailyRecords.find((r) => r.date === today);
    if (existingRecord) {
      existingRecord.minutes += mins;
    } else {
      config.state.dailyRecords.push({
        date: today,
        minutes: config.state.todayMinutes,
        words: config.state.todayWords,
      });
    }

    if (config.state.todayMinutes > 0) {
      addToStreakHistory(today);
    }

    await checkAndUpdateStreak();
  }

  async function addReadingWords(words: number): Promise<void> {
    const today = getTodayString();
    checkTodayReset();

    config.state.todayWords += words;
    config.state.lastReadDate = today;

    const existingRecord = config.state.dailyRecords.find((r) => r.date === today);
    if (existingRecord) {
      existingRecord.words += words;
    } else {
      config.state.dailyRecords.push({
        date: today,
        minutes: config.state.todayMinutes,
        words: config.state.todayWords,
      });
    }

    if (config.state.todayWords > 0) {
      addToStreakHistory(today);
    }

    await checkAndUpdateStreak();
  }

  function addToStreakHistory(date: string) {
    if (!config.state.streakHistory.includes(date)) {
      config.state.streakHistory.push(date);
      config.state.streakHistory.sort();
    }
  }

  function isTodayGoalMet(): boolean {
    if (goalType.value === "minutes") {
      return todayMinutes.value >= goalMinutes.value;
    } else {
      return todayWords.value >= goalWords.value;
    }
  }

  async function checkAndUpdateStreak(): Promise<void> {
    const today = getTodayString();
    const yesterday = getYesterdayString();

    if (!config.state.streakHistory.length) {
      config.state.streakDays =
        config.state.todayMinutes > 0 || config.state.todayWords > 0 ? 1 : 0;
      return;
    }

    const hasReadToday = config.state.todayMinutes > 0 || config.state.todayWords > 0;
    const hasReadYesterday = config.state.streakHistory.includes(yesterday);

    if (hasReadToday) {
      if (hasReadYesterday) {
        const lastDate = config.state.streakHistory[config.state.streakHistory.length - 1];
        if (lastDate !== today) {
          config.state.streakDays += 1;
        }
      } else {
        config.state.streakDays = 1;
      }
    }
  }

  function getDailyRecord(date: string): DailyRecord | undefined {
    return config.state.dailyRecords.find((r) => r.date === date);
  }

  function getMonthCalendar(
    year: number,
    month: number,
  ): {
    date: string;
    hasRead: boolean;
    intensity: 0 | 1 | 2 | 3;
    isStreak: boolean;
  }[] {
    const result: {
      date: string;
      hasRead: boolean;
      intensity: 0 | 1 | 2 | 3;
      isStreak: boolean;
    }[] = [];

    const firstDay = new Date(year, month - 1, 1);
    const lastDay = new Date(year, month, 0);
    const startDayOfWeek = firstDay.getDay();
    const daysInMonth = lastDay.getDate();

    for (let i = 0; i < startDayOfWeek; i++) {
      result.push({ date: "", hasRead: false, intensity: 0, isStreak: false });
    }

    const sortedHistory = [...config.state.streakHistory].sort();
    let currentStreak = 0;
    let prevDate: Date | null = null;

    for (let day = 1; day <= daysInMonth; day++) {
      const dateStr = `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
      const record = getDailyRecord(dateStr);
      const hasRead = !!record && (record.minutes > 0 || record.words > 0);

      let intensity: 0 | 1 | 2 | 3 = 0;
      if (hasRead && record) {
        if (goalType.value === "minutes") {
          const minutes = record.minutes;
          if (minutes >= 30) intensity = 3;
          else if (minutes >= 16) intensity = 2;
          else if (minutes >= 1) intensity = 1;
        } else {
          const words = record.words;
          if (words >= 3000) intensity = 3;
          else if (words >= 1500) intensity = 2;
          else if (words >= 1) intensity = 1;
        }
      }

      const currentDate = new Date(year, month - 1, day);
      let isStreak = false;
      if (hasRead) {
        if (prevDate) {
          const diffDays = Math.floor(
            (currentDate.getTime() - prevDate.getTime()) / (1000 * 60 * 60 * 24),
          );
          if (diffDays === 1) {
            currentStreak++;
          } else {
            currentStreak = 1;
          }
        } else {
          currentStreak = 1;
        }
        isStreak = currentStreak >= 3;
      } else {
        currentStreak = 0;
      }

      prevDate = currentDate;
      result.push({ date: dateStr, hasRead, intensity, isStreak });
    }

    return result;
  }

  function getWeekStats(): { daysRead: number; totalMinutes: number; totalWords: number } {
    const today = getTodayString();
    const weekAgo = new Date();
    weekAgo.setDate(weekAgo.getDate() - 7);
    const weekAgoStr = `${weekAgo.getFullYear()}-${String(weekAgo.getMonth() + 1).padStart(2, "0")}-${String(weekAgo.getDate()).padStart(2, "0")}`;

    const weekRecords = config.state.dailyRecords.filter(
      (r) => r.date >= weekAgoStr && r.date <= today,
    );

    const daysRead = weekRecords.filter((r) => r.minutes > 0 || r.words > 0).length;
    const totalMinutes = weekRecords.reduce((sum, r) => sum + r.minutes, 0);
    const totalWords = weekRecords.reduce((sum, r) => sum + r.words, 0);

    return { daysRead, totalMinutes, totalWords };
  }

  async function resetToday(): Promise<void> {
    config.state.todayMinutes = 0;
    config.state.todayWords = 0;
  }

  async function resetAll(): Promise<void> {
    config.state.todayMinutes = 0;
    config.state.todayWords = 0;
    config.state.streakDays = 0;
    config.state.lastReadDate = "";
    config.state.streakHistory = [];
    config.state.dailyRecords = [];
  }

  return {
    goalMinutes,
    goalType,
    goalWords,
    todayMinutes,
    todayWords,
    streakDays,
    lastReadDate,
    streakHistory,
    dailyRecords,
    progressPercentage,
    setGoalType,
    setGoalMinutes,
    setGoalWords,
    addReadingMinutes,
    addReadingWords,
    isTodayGoalMet,
    checkAndUpdateStreak,
    getDailyRecord,
    getMonthCalendar,
    getWeekStats,
    resetToday,
    resetAll,
  };
});
